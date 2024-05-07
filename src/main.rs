pub use self::error::{Error, Result};

use crate::model::ModelController;

use axum::response::{IntoResponse, Response};
use axum::routing::get_service;
use axum::{middleware, Json, Router};
use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use uuid::Uuid;

mod env_config;
mod error;
mod links_db;
mod model;
mod web;

// Don't need to serve any static files just now but in case we have a need they can go in
// ./docroot/static and be served under URL /static.
fn routes_static() -> Router {
    Router::new().nest_service("/static", get_service(ServeDir::new("./docroot/static/")))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Get the database URL and listen address(es) from the environment.
    let conf = env_config::load();
    //println!("{:#?}", conf);

    let db_pool = SqlitePoolOptions::new()
        .max_lifetime(None)
        .idle_timeout(None)
        .connect(&conf.db_url)
        .await
        .unwrap();

    // Put the SQLite handle into the model controller so it can be passed to all route handlers.
    let mc = ModelController::new(db_pool.clone()).await?;

    // Create database contents - we're using in-memory SQLite so it disappears on every restart.
    links_db::populate(&db_pool).await.unwrap();

    let routes_all = Router::new()
        .nest("/api", web::routes_api::routes(mc.clone()))
        .layer(middleware::map_response(main_response_mapper))
        .fallback_service(routes_static());

    let listener = TcpListener::bind(&conf.listen_on).await.unwrap();

    println!(
        "->> Listening on http://{:?}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn main_response_mapper(res: Response) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");

    let uuid = Uuid::new_v4();

    // Get the eventual error.
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // If this error is one for the client, build a new resoonse to show that.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error, descr)| {
            let client_error_body = json!({
                "error": {
                    "req_uuid": uuid.to_string(),
                    "type": client_error.as_ref(),
                    "description": descr,
                }
            });

            println!("    ->> client_error_body: {client_error_body}");

            (*status_code, Json(client_error_body)).into_response()
        });

    println!("    ->> server log line - {uuid} - Error: {service_error:?}");

    println!();

    error_response.unwrap_or(res)
}
