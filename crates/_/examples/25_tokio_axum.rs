use anput::bundle::DynamicBundle;
use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    serve,
};
use keket::{
    database::{AssetDatabase, path::AssetPathStatic},
    fetch::{AssetBytesAreReadyToProcess, future::FutureAssetFetch},
    protocol::{bytes::BytesAssetProtocol, text::TextAssetProtocol},
    third_party::anput::component::Component,
};
use std::{error::Error, path::PathBuf, sync::Arc};
use tokio::{
    net::TcpListener,
    sync::RwLock,
    time::{Duration, sleep},
};

async fn tokio_load_file_bundle(path: AssetPathStatic) -> Result<DynamicBundle, Box<dyn Error>> {
    let file_path = PathBuf::from("resources").join(path.path());

    let bytes = tokio::fs::read(&file_path).await?;

    let mut bundle = DynamicBundle::default();
    bundle
        .add_component(AssetBytesAreReadyToProcess(bytes))
        .map_err(|_| format!("Failed to add bytes to bundle for asset: {}", path))?;
    Ok(bundle)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        .with_fetch(FutureAssetFetch::new(tokio_load_file_bundle));
    let database = Arc::new(RwLock::new(database));
    let database2 = database.clone();

    tokio::spawn(async move {
        loop {
            if let Err(error) = database2.write().await.maintain() {
                eprintln!("Error maintaining database: {}", error);
            }
            sleep(Duration::from_millis(100)).await;
        }
    });

    let app = Router::new()
        .route("/bytes/{*asset_path}", get(serve_asset_bytes_handler))
        .route("/text/{*asset_path}", get(serve_asset_text_handler))
        .with_state(database);

    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("Listening on {}", listener.local_addr()?);

    serve(listener, app).await?;

    Ok(())
}

async fn serve_asset_bytes_handler(
    Path(asset_path): Path<String>,
    State(database): State<Arc<RwLock<AssetDatabase>>>,
) -> impl IntoResponse {
    println!("Received request for bytes asset: {}", asset_path);

    match get_asset::<Vec<u8>>(format!("bytes://{asset_path}"), database).await {
        Ok(bytes) => Response::builder()
            .header("Content-Type", "application/octet-stream")
            .body(Body::from(bytes))
            .unwrap(),
        Err(error) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain")
            .body(Body::from(error))
            .unwrap(),
    }
}

async fn serve_asset_text_handler(
    Path(asset_path): Path<String>,
    State(database): State<Arc<RwLock<AssetDatabase>>>,
) -> impl IntoResponse {
    println!("Received request for text asset: {}", asset_path);

    match get_asset::<String>(format!("text://{asset_path}"), database).await {
        Ok(bytes) => Response::builder()
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Body::from(bytes))
            .unwrap(),
        Err(error) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain")
            .body(Body::from(error))
            .unwrap(),
    }
}

async fn get_asset<T: Component + Clone>(
    path: impl Into<AssetPathStatic>,
    database: Arc<RwLock<AssetDatabase>>,
) -> Result<T, String> {
    let path = path.into();

    let handle = database
        .write()
        .await
        .ensure(path.clone())
        .map_err(|e| e.to_string())?;

    while !handle.is_ready_to_use(&*database.read().await) {
        sleep(Duration::from_millis(10)).await;
    }

    handle
        .access_checked::<&T>(&*database.read().await)
        .cloned()
        .ok_or_else(|| format!("Asset has no bytes: {path}"))
}
