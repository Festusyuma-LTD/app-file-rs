use crate::controllers;
use crate::state::ServiceState;
use crate::util::middleware::disabled_routes;
use axum::middleware;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub fn app<S>(state: Arc<ServiceState>) -> OpenApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    OpenApiRouter::new()
        .nest(
            "/upload",
            OpenApiRouter::new()
                .routes(routes!(controllers::upload::init_upload))
                .routes(routes!(controllers::upload::get_url)),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            disabled_routes,
        ))
        .with_state(state)
}
