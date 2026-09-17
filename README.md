# file

Library crate for issuing presigned/signed URLs against an S3-compatible object storage
bucket, optionally fronted by a CloudFront CDN — for clients that need to upload/download
files directly to/from storage without proxying bytes through the app server.

## What it does

- **Uploads**: generates a presigned S3 `PUT` URL a client can upload directly to.
- **Reads**: generates a time-limited URL for reading a previously uploaded file — a
  CloudFront signed URL if a CDN key is configured, otherwise a presigned S3 `GET` URL.

## Wiring it into an app

```rust
use file::config::ConfigBuilder;

let config = ConfigBuilder::new()
    .region("us-east-1")
    .bucket("my-bucket")
    .base_url("https://cdn.example.com")
    // Only if reads should go through a CloudFront signed URL instead of a
    // presigned S3 GET. Omit to always use presigned S3 URLs.
    .cdn_key("CLOUDFRONT_KEY_PAIR_ID", "/path/to/private_key.pem")
    .build()
    .await;

let upload_service = file::UploadService::new(config);
let state = Arc::new(file::state::ServiceState {
    upload_service,
    disabled_routes: Default::default(),
});

// `file::app` returns a `utoipa_axum::router::OpenApiRouter`, so its OpenAPI paths/schemas
// can be merged into the rest of the app's spec before splitting into a plain `axum::Router`.
use utoipa_axum::router::OpenApiRouter;

let (router, openapi) = OpenApiRouter::new()
    .nest("/file", file::app(state))
    .split_for_parts();

let app: axum::Router = router; // serve `openapi` (e.g. via utoipa-swagger-ui) however you like
```

If you don't care about the OpenAPI spec, `OpenApiRouter<S>` also converts directly into
`axum::Router<S>` via `Into`, so `axum::Router::new().nest("/file", file::app(state).into())`
works too.

## Routes

Mounted under whatever prefix the parent app nests `file::app(state)` at, then under
`/upload`:

| Method | Path                  | Request body                     | Response                            |
| ------ | --------------------- | --------------------------------- | ------------------------------------ |
| POST   | `/upload/init_upload` | `GenerateUploadUrlRequest` (JSON) | 201 `GenerateUploadUrlResponse`      |
| GET    | `/upload/url`         | —                                  | presigned/signed URL string          |

`GenerateUploadUrlRequest`:

```json
{ "name": "photo.jpg", "mimetype": "image/jpeg", "expiration": 3600 }
```

`mimetype` and `expiration` are optional (`expiration` defaults to 60 seconds).
`GenerateUploadUrlResponse` is `{ "upload_url": "...", "url": "..." }` — `upload_url` is
the presigned URL to `PUT` the file to, `url` the resulting public URL once uploaded.

Each handler carries a `#[utoipa::path]` annotation, so the routes and their request/response
schemas show up in the `utoipa::openapi::OpenApi` returned alongside the router.

Individual routes can be disabled per-app via `DisabledRoutes`/`Route` (see
`util::routes`) — a disabled route responds `403`.

> **Note:** `GET /upload/url` currently ignores the request entirely and always returns a
> signed URL for the fixed key `public/age.txt` with a 300s expiration
> (`controllers/upload.rs`) — it's a placeholder, not yet a general "get URL for this key"
> endpoint.

## Config

Built via `file::config::ConfigBuilder`:

| Method       | Required | Description                                                          |
| ------------ | -------- | ---------------------------------------------------------------------|
| `.region()`  | yes      | AWS region for the S3 client.                                        |
| `.bucket()`  | yes      | S3 bucket name.                                                      |
| `.base_url()`| yes      | Public base URL files are served from (CDN or bucket origin).        |
| `.cdn_key()` | no       | CloudFront key pair ID + path to the private key PEM, for signed CDN reads. |

`.build()` panics (via `assert!`) if `region`, `bucket`, or `base_url` weren't set.
