use crate::dto;
use crate::util::state::ServiceStateType;
use axum::Json;
use axum::extract::State;
use shared::error::ServiceError;
use shared::response::ServiceResponse;

#[utoipa::path(
    post,
    path = "/init_upload",
    tag = "upload",
    request_body = dto::upload::GenerateUploadUrlRequest,
    responses(
        (status = 201, description = "Presigned upload URL and resulting public URL", body = dto::upload::GenerateUploadUrlResponse),
        (status = 500, description = "Failed to generate the presigned URL"),
        (status = 403, description = "Route disabled for this deployment"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/url",
    tag = "upload",
    responses(
        (status = 200, description = "Presigned/signed URL for reading the file", body = String),
        (status = 403, description = "Route disabled for this deployment"),
    )
)]
pub async fn get_url(State(state): ServiceStateType) -> ServiceResponse<String> {
    state
        .upload_service
        .generate_url("public/age.txt", 300)
        .await
        .into()
}
