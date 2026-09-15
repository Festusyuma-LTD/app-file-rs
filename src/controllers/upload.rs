use crate::dto;
use crate::util::state::ServiceStateType;
use axum::Json;
use axum::extract::State;
use shared::error::AppError;
use shared::response::Response;

pub async fn init_upload(
    State(state): ServiceStateType,
    Json(payload): Json<dto::upload::GenerateUploadUrlRequest>,
) -> Response<dto::upload::GenerateUploadUrlResponse> {
    state
        .upload_service
        .generate_upload_url(payload)
        .await
        .map(|r| Response::Status(r, 201))
        .unwrap_or(AppError::ServerError.into())
}

pub async fn get_url(State(state): ServiceStateType) -> Response<String> {
    state
        .upload_service
        .generate_url("public/age.txt", 300)
        .await
        .into()
}
