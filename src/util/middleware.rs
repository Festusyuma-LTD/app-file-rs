use crate::util::state::ServiceState;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use shared::error::AppError;
use std::sync::Arc;

pub async fn disabled_routes(
    State(state): State<Arc<ServiceState>>,
    req: Request,
    next: Next,
) -> Response {
    let disabled = state.disabled_routes.is_disabled(req.uri().path());

    if disabled {
        return AppError::HttpMessage(403, "this route is disabled".into()).into_response();
    }

    next.run(req).await
}
