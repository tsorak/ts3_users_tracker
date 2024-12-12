use axum::{
    self, Router,
    extract::State,
    response::{Html, IntoResponse, Response},
    routing::get,
};

use crate::ArcOnlineUsers;

pub async fn start(
    users: ArcOnlineUsers,
) -> anyhow::Result<axum::serve::Serve<axum::Router, axum::Router>> {
    let router = Router::new().route("/", get(handler));

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|err| anyhow::anyhow!("Failed to bind listener on '{addr}', {err}"))?;
    Ok(axum::serve(listener, router.with_state(users)))
}

async fn handler(state: State<ArcOnlineUsers>) -> Response {
    let lock = state.lock().await;
    let status = lock.get_status_display();

    let html = format!(
        "
        <html>
            <head></head>
            <body>
                <pre>
{status}
                </pre>
            </body>
        </html>
        "
    );

    Html(html).into_response()
}
