use crate::config::Config;
use crate::dto;
use crate::util::error::ServiceResult;
use anyhow::Error;
use aws_sdk_s3::presigning;
use cloudfront_sign::SignedOptions;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;

/// Issues presigned/signed URLs for uploading to and reading from the configured
/// object storage bucket (and, optionally, a CloudFront-fronted CDN).
pub struct UploadService {
    config: Arc<Config>,
}

impl UploadService {
    /// Constructs an `UploadService` backed by the given [`Config`] (S3 client, bucket,
    /// base URL, and optional CDN signing key).
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl UploadService {
    /// Generates a presigned URL for uploading a file to an object storage bucket.
    ///
    /// This method creates a presigned URL that allows a client to perform a
    /// PUT operation to upload a file directly to the storage bucket. The URL
    /// will expire after the specified duration.
    ///
    /// # Parameters
    /// - `payload`: A [`dto::upload::GenerateUploadUrlRequest`] with:
    ///   - `name`: The name of the file to be uploaded, used to generate the file's key in the bucket.
    ///   - `mimetype`: An optional MIME type for the file. If `None`, no content type will be set.
    ///   - `expiration`: An optional duration (in seconds) for which the presigned URL will remain valid.
    ///     Defaults to `60` seconds if `None`.
    ///
    /// # Returns
    /// - `Ok(GenerateUploadUrlResponse)`: Contains `upload_url`, the presigned URL to `PUT` the file to,
    ///   and `url`, the resulting public URL of the file once uploaded.
    /// - `Err(AppError)`: An error if URL generation fails due to configuration issues, invalid inputs,
    ///   or unexpected failures in the presigning process.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The presigning duration is invalid (e.g. cannot be represented, per `PresigningConfig::expires_in`).
    /// - The process of creating a presigned URL fails (e.g., due to invalid keys or network issues).
    ///
    /// As with [`Self::generate_url`], the underlying failure is logged and collapsed to
    /// `AppError::ServerError` rather than surfaced to the caller.
    ///
    /// # Example
    /// ```rust
    /// use file::UploadService;
    /// use file::dto::upload::GenerateUploadUrlRequest;
    ///
    /// # async fn example(upload_service: UploadService) {
    /// let payload = GenerateUploadUrlRequest {
    ///     name: "example.txt".to_string(),
    ///     mimetype: Some("text/plain".to_string()),
    ///     expiration: Some(3600),
    /// };
    ///
    /// match upload_service.generate_upload_url(payload).await {
    ///     Ok(response) => println!("Presigned URL: {}", response.upload_url),
    ///     Err(e) => eprintln!("Error generating upload URL: {}", e),
    /// }
    /// # }
    /// ```
    ///
    /// # Preconditions
    /// - `UploadService` must be constructed via `UploadService::new` with a fully initialized `Config`
    ///   (valid S3 `client` and `bucket`).
    ///
    /// # Notes
    /// - The expiration duration is subject to the limits or constraints imposed by the object storage services.
    ///   Ensure that `expiration` is within the allowed range.
    /// - The returned `upload_url` must be used before it expires; otherwise, access will be denied.
    ///
    /// # Dependencies
    /// This method relies on:
    /// - `dto::object::FileObject` for deriving the object key and content type from `name`/`mimetype`.
    /// - `presigning::PresigningConfig` for configuring the presign duration.
    /// - `self.config` (the `Config` this `UploadService` was constructed with) for the storage client and bucket.
    pub async fn generate_upload_url(
        &self,
        payload: dto::upload::GenerateUploadUrlRequest,
    ) -> ServiceResult<dto::upload::GenerateUploadUrlResponse> {
        let file =
            dto::object::FileObject::from_filename(&payload.name, payload.mimetype.as_deref());

        let expires_in = Duration::from_secs(payload.expiration.unwrap_or(60));
        let presigning_config =
            presigning::PresigningConfig::expires_in(expires_in).map_err(Error::from)?;

        let presigned_put = self
            .config
            .client
            .put_object()
            .bucket(&self.config.bucket)
            .key(file.key)
            .content_type(file.mimetype)
            .presigned(presigning_config)
            .await
            .map_err(Error::from)?;

        Ok(dto::upload::GenerateUploadUrlResponse {
            upload_url: presigned_put.uri().into(),
            url: file.url.unwrap_or(String::new()),
        })
    }

    /// Produces a time-limited URL for reading an already-uploaded file.
    ///
    /// If the services was configured with a CDN signing key ([`Config::cdn_key`]), this
    /// signs the file's CDN URL using CloudFront's canned-policy scheme
    /// (`cloudfront_sign::get_signed_url`). Otherwise it falls back to a presigned S3
    /// `GET` URL, the same mechanism [`Self::generate_upload_url`] uses for `PUT`.
    ///
    /// # Parameters
    /// - `url`: The file's key/path or previously-issued URL, as returned in
    ///   [`dto::upload::GenerateUploadUrlResponse::url`]. Resolved to an object key via
    ///   [`dto::object::FileObject::from_url`].
    /// - `expires_in_seconds`: How long the returned URL stays valid.
    ///
    /// # Returns
    /// - `Ok(String)`: The signed CDN URL or presigned S3 URL, depending on configuration.
    /// - `Err(AppError)`: If `url` cannot be parsed, the CDN private key file cannot be
    ///   read, signing fails, or the presigning/S3 call fails.
    ///
    /// # Notes
    /// - Errors are not surfaced with detail to callers: any underlying failure is logged
    ///   and collapsed to `AppError::ServerError` by the `From<anyhow::Error> for AppError`
    ///   conversion.
    pub async fn generate_url(&self, url: &str, expires_in_seconds: u64) -> ServiceResult<String> {
        let file = dto::object::FileObject::from_url(url)?;

        let url: String = if let Some(cdn_key) = &self.config.cdn_key {
            let resource_url = format!("{}/{}", &self.config.base_url, &file.key);

            let private_key = fs::read_to_string(&cdn_key.private_key_path)
                .await
                .map_err(Error::from)?;

            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(Error::from)?
                .as_secs();

            let expires_at = current_time + expires_in_seconds;

            let options = SignedOptions {
                key_pair_id: Cow::from(&cdn_key.key_id),
                private_key: Cow::from(private_key),
                resource: Some(Cow::from(&resource_url)),
                date_less_than: expires_at,
                ..Default::default()
            };

            cloudfront_sign::get_signed_url(&resource_url, &options).map_err(Error::from)?
        } else {
            let expires_in = Duration::from_secs(expires_in_seconds);

            let presigning_config =
                presigning::PresigningConfig::expires_in(expires_in).map_err(Error::from)?;

            let presigned_put = self
                .config
                .client
                .get_object()
                .bucket(&self.config.bucket)
                .key(file.key)
                .presigned(presigning_config)
                .await
                .map_err(Error::from)?;

            presigned_put.uri().to_string()
        };

        Ok(url)
    }
}
