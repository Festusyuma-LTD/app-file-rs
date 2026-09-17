use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct GenerateUploadUrlRequest {
    pub name: String,
    pub mimetype: Option<String>,
    pub expiration: Option<u64>,
}

#[derive(Serialize, ToSchema)]
pub struct GenerateUploadUrlResponse {
    pub upload_url: String,
    pub url: String,
}

#[derive(Deserialize)]
struct GenerateUrlRequest {}
