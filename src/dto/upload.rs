use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GenerateUploadUrlRequest {
    pub name: String,
    pub mimetype: Option<String>,
    pub expiration: Option<u64>,
}

#[derive(Serialize)]
pub struct GenerateUploadUrlResponse {
    pub upload_url: String,
    pub url: String,
}

#[derive(Deserialize)]
struct GenerateUrlRequest {}
