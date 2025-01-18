use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use notify::{Config, Event, PollWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use serde::{Deserialize, Serialize};
use std::{
    fs::create_dir_all,
    io::{stderr, stdout},
    net::SocketAddr,
    process::{exit, Command},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::spawn,
    time::Duration,
};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};
use warp::{
    filters::ws::{Message, WebSocket},
    http::Response,
    reject::Reject,
    Filter, Rejection, Reply,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum PipelineCommand {
    Terminate,
    Execute {
        program: String,
        arguments: Vec<String>,
    },
}

impl PipelineCommand {
    fn run(&self, path: String) {
        match self {
            Self::Terminate => {
                exit(0);
            }
            Self::Execute { program, arguments } => {
                let _ = Command::new(program)
                    .arg(path)
                    .args(arguments)
                    .stdout(stdout())
                    .stderr(stderr())
                    .status();
            }
        }
    }
}

#[derive(Debug)]
struct MessageError(#[allow(dead_code)] pub String);

impl Reject for MessageError {}

#[derive(Default, Clone)]
struct ChangeBindings {
    senders: Arc<Mutex<Vec<Sender<String>>>>,
}

impl ChangeBindings {
    fn receiver(&self) -> Receiver<String> {
        let (sender, receiver) = channel();
        self.senders.lock().unwrap().push(sender);
        receiver
    }

    fn send(&self, path: &str) {
        let mut senders = self.senders.lock().unwrap();
        senders.retain(|sender| sender.send(path.to_owned()).is_ok());
    }
}

async fn get_file_handler(path: String) -> Result<impl Reply, Rejection> {
    let file_path = std::env::current_dir().unwrap().join(path);

    if !file_path.exists() {
        return Err(warp::reject::not_found());
    }

    let file_bytes = tokio::fs::read(&file_path).await.unwrap();

    println!("* Requested file: {:?}", file_path);
    Ok(Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(file_bytes)
        .unwrap())
}

async fn put_file_handler(path: String, body: Bytes) -> Result<impl Reply, Rejection> {
    let file_path = std::env::current_dir().unwrap().join(path);

    if let Some(parent) = file_path.parent() {
        create_dir_all(parent).unwrap();
    }

    let mut file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .open(&file_path)
        .await
        .unwrap();

    file.write_all(&body).await.unwrap();

    println!("* Created file: {:?}", file_path);
    Ok(warp::reply::with_status(
        "File saved",
        warp::http::StatusCode::OK,
    ))
}

async fn delete_file_handler(path: String) -> Result<impl Reply, Rejection> {
    let file_path = std::env::current_dir().unwrap().join(path);

    if !file_path.exists() {
        return Err(warp::reject::not_found());
    }

    if let Err(err) = fs::remove_file(&file_path).await {
        return Err(warp::reject::custom(MessageError(format!("{}", err))));
    }

    println!("* Deleted file: {:?}", file_path);
    Ok(warp::reply::with_status(
        "File deleted",
        warp::http::StatusCode::OK,
    ))
}

async fn run_command_handler(path: String, body: PipelineCommand) -> Result<impl Reply, Rejection> {
    body.run(path);
    println!("* Executed command: {:?}", body);
    Ok(warp::reply::with_status(
        "Executed command",
        warp::http::StatusCode::OK,
    ))
}

async fn client_connected(ws: WebSocket, id: usize, receiver: Receiver<String>) {
    println!("* WebSocket client connected:{}", id);
    let (mut client_tx, _) = ws.split();
    loop {
        let mut disconnected = false;
        while let Ok(path) = receiver.try_recv() {
            println!("* WebSocket sent changed path: {:?} to: {}", path, id);
            if client_tx.send(Message::text(path)).await.is_err() {
                disconnected = true;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        if client_tx
            .send(Message::ping(id.to_be_bytes()))
            .await
            .is_err()
        {
            disconnected = true;
        }
        if disconnected {
            let _ = client_tx.close().await;
            println!("* WebSocket client disconnected: {}", id);
            return;
        }
    }
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    args.next();
    let address = args.next().unwrap_or("127.0.0.1:8080".to_owned());

    println!(
        "* Start file system watcher at path: {:?}",
        std::env::current_dir().unwrap()
    );
    let (watcher_tx, watcher_rx) = channel::<NotifyResult<Event>>();
    let mut watcher = PollWatcher::new(
        watcher_tx.clone(),
        Config::default().with_poll_interval(Duration::from_secs(10)),
    )
    .unwrap();
    watcher
        .watch(&std::env::current_dir().unwrap(), RecursiveMode::Recursive)
        .unwrap();

    let bindings = ChangeBindings::default();
    let bindings2 = bindings.clone();
    spawn(move || {
        let current_dir = format!(
            "{}{}",
            std::env::current_dir().unwrap().to_string_lossy(),
            std::path::MAIN_SEPARATOR
        );
        for event in watcher_rx.into_iter().flatten() {
            if event.kind.is_modify() {
                for path in event.paths {
                    println!("* File changed: {:?}", path);
                    let path = path.to_string_lossy();
                    let path = path.as_ref();
                    bindings2.send(path.strip_prefix(&current_dir).unwrap_or(path));
                }
            }
        }
    });

    println!("* Start asset server at address: {:?}", address);
    let id_generator = Arc::new(AtomicUsize::default());
    warp::serve(
        warp::path!("assets" / String)
            .and(warp::get())
            .and_then(get_file_handler)
            .or(warp::path!("assets" / String)
                .and(warp::put())
                .and(warp::body::bytes())
                .and_then(put_file_handler))
            .or(warp::path!("assets" / String)
                .and(warp::delete())
                .and_then(delete_file_handler))
            .or(warp::path!("run" / String)
                .and(warp::post())
                .and(warp::body::json())
                .and_then(run_command_handler))
            .or(warp::path("changes")
                .and(warp::ws())
                .and(warp::any().map(move || id_generator.fetch_add(1, Ordering::Relaxed)))
                .and(warp::any().map(move || bindings.clone()))
                .map(
                    move |ws: warp::ws::Ws, id: usize, bindings: ChangeBindings| {
                        println!("* WebSocket new client connection: {}", id);
                        ws.on_upgrade(move |ws| client_connected(ws, id, bindings.receiver()))
                    },
                )),
    )
    .run(
        address
            .parse::<SocketAddr>()
            .unwrap_or_else(|error| panic!("Invalid IP address: {}. Error: {}", address, error)),
    )
    .await;
}
