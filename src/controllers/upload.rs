use crate::dto;
use crate::util::state::ServiceStateType;
use axum::Json;
use axum::extract::State;
use shared::error::ServiceError;
use shared::response::ServiceResponse;

pub async fn init_upload(
    State(state): ServiceStateType,
    Json(payload): Json<dto::upload::GenerateUploadUrlRequest>,
) -> ServiceResponse<dto::upload::GenerateUploadUrlResponse> {
    state
        .upload_service
        .generate_upload_url(payload)
        .await
        .map(|r| ServiceResponse::Status(r, 201))
        .unwrap_or(ServiceError::ServerError.into())
}

pub async fn get_url(State(state): ServiceStateType) -> ServiceResponse<String> {
    state
        .upload_service
        .generate_url("public/age.txt", 300)
        .await
        .into()
}
