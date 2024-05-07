use crate::model::{Link, LinkForCreate, ModelController};
use crate::Result;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

pub fn routes(mc: ModelController) -> Router {
    Router::new()
        .route("/link/:short", get(get_link))
        .route("/link", post(create_link))
        .route("/all", get(list_links)) // For debug only. Really enumerating all links is not desirable.
        .with_state(mc)
}

// GET to /api/link/:short to retrieve the mapping to the longer target, if it exists.
async fn get_link(
    State(mc): State<ModelController>,
    Path(short): Path<String>,
) -> Result<Json<Link>> {
    println!("->> {:<12} - get_link", "HANDLER");

    // TODO: Check that `short` is composed of only characters from the Base58 alphabet and just
    // return 404 straight away if not.

    let link = mc.get_link(&short).await?;
    /*
        let link = Link {
            uuid: "foo".to_string(),
            short: short,
            target: "https://example.com/".to_string(),
        };
    */

    Ok(Json(link))
}

// POST to /api/link to create a new snort link.
async fn create_link(
    State(mc): State<ModelController>,
    Json(link_fc): Json<LinkForCreate>,
) -> Result<(StatusCode, Json<Link>)> {
    println!("->> {:<12} - create_link", "HANDLER");

    let link = mc.create_link(link_fc).await?;

    Ok((StatusCode::CREATED, Json(link)))
}

// GET /api/all for a list of all short links (for debugging only).
async fn list_links(State(mc): State<ModelController>) -> Result<Json<Vec<Link>>> {
    println!("->> {:<12} - list_links", "HANDLER");

    let links = mc.list_links().await?;

    Ok(Json(links))
}
