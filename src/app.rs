use crate::controllers;
use crate::state::ServiceState;
use crate::util::middleware::disabled_routes;
use axum::Router;
use axum::middleware;
use axum::routing::{get, post};
use std::sync::Arc;

pub fn app<S>(state: Arc<ServiceState>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .nest(
            "/upload",
            Router::new()
                .route("/init_upload", post(controllers::upload::init_upload))
                .route("/url", get(controllers::upload::get_url)),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            disabled_routes,
        ))
        .with_state(state)
}
