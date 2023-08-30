use std::{net::SocketAddr, sync::Arc};

use axum::{routing::get, Extension, Router};
use common::error::Error;
use tokio::sync::{watch, Notify};
use serde::Serialize;

#[derive(Clone)]
struct AppState {
    notify_for_start: Arc<Notify>,
    started_receiver: watch::Receiver<bool>,
}

pub async fn start(
    addr: SocketAddr,
    notify_for_start: Arc<Notify>,
    started_receiver: watch::Receiver<bool>,
) -> Result<(), Error> {
    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_origin(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let state = AppState {
        notify_for_start,
        started_receiver,
    };

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/start", get(start_handler))
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

#[derive(Debug, Serialize)]
struct StartResponse {
    success: bool,
    msg: &'static str,
}

async fn start_handler(Extension(state): Extension<AppState>) -> String {
    state.notify_for_start.notify_waiters();
    let started = { *state.started_receiver.borrow() };
    let response = StartResponse {
        success: !started,
        msg: if started { "already started" } else { "success" },
    };
    serde_json::to_string(&response).unwrap()
}
