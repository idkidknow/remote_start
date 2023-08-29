use std::{net::SocketAddr, sync::Arc};

use axum::{Router, routing::{get, post}, Extension};
use common::error::Error;
use tokio::sync::Notify;

#[derive(Clone)]
struct AppState {
    notify_for_start: Arc<Notify>,
}

pub async fn start(addr: SocketAddr, notify_for_start: Arc<Notify>) -> Result<(), Error> {
    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_origin(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let state = AppState {
        notify_for_start
    };

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/start", post(start_handler))
        .layer(cors)
        .layer(Extension(state));
    let res = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await;
    if let Err(e) = res {
        return Err(Error::NetworkError(e.to_string()));
    }
    Ok(())
}

async fn root_handler() -> &'static str {
    "/start"
}

async fn start_handler(Extension(state): Extension<AppState>) -> String {
    state.notify_for_start.notify_one();
    "Ok".to_owned()
}
