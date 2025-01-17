use bytes::Bytes;
use crossbeam_channel::bounded;
use futures_util::{SinkExt, StreamExt};
use notify::{Config, Event, PollWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use serde::{Deserialize, Serialize};
use std::{
    fs::create_dir_all,
    io::{stderr, stdout},
    net::SocketAddr,
    process::{exit, Command},
    sync::mpsc::channel,
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

async fn post_file_handler(path: String, body: PipelineCommand) -> Result<impl Reply, Rejection> {
    body.run(path);
    println!("* Executed command: {:?}", body);
    Ok(warp::reply::with_status(
        "Executed command",
        warp::http::StatusCode::OK,
    ))
}

async fn client_connected(ws: WebSocket, rx: crossbeam_channel::Receiver<String>) {
    let (mut client_tx, _) = ws.split();

    while let Ok(path) = rx.recv() {
        let _ = client_tx.send(Message::text(path)).await;
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

    let (changes_tx, changes_rx) = bounded(100);
    let changes_rx = warp::any().map(move || changes_rx.clone());
    spawn(move || {
        for event in watcher_rx.into_iter().flatten() {
            if event.kind.is_modify() {
                for path in event.paths {
                    println!("* File changed: {:?}", path);
                    let _ = changes_tx.send(path.to_string_lossy().as_ref().to_owned());
                }
            }
        }
    });

    println!("* Start asset server at address: {:?}", address);
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
            .or(warp::path!("assets" / String)
                .and(warp::post())
                .and(warp::body::json())
                .and_then(post_file_handler))
            .or(warp::path("changes").and(warp::ws()).and(changes_rx).map(
                move |ws: warp::ws::Ws, rx: crossbeam_channel::Receiver<String>| {
                    ws.on_upgrade(|ws| client_connected(ws, rx))
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
