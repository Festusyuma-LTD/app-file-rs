use crate::util::error::ServiceResult;

use anyhow::Error;
use serde::Serialize;
use std::path::Path;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[derive(Serialize)]
pub struct FileObject {
    pub key: String,
    pub mimetype: String,
}

impl FileObject {
    pub(crate) fn new(key: String, mimetype: String) -> FileObject {
        FileObject { key, mimetype }
    }

    pub(crate) fn from_filename(file_name: &str, mimetype: Option<&str>) -> FileObject {
        let path = Path::new(file_name);

        let mimetype = mimetype
            .map(|v| mime_guess::Mime::from_str(v).ok())
            .unwrap_or_else(|| mime_guess::from_path(file_name).first())
            .unwrap_or(mime_guess::mime::APPLICATION_OCTET_STREAM);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let file_key = path.with_extension("");
        let file_key = file_key.to_str().unwrap_or(file_name);
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        let key = if ext.is_empty() {
            format!("{file_key}-{timestamp}")
        } else {
            format!("{file_key}-{timestamp}.{ext}")
        };

        FileObject {
            key,
            mimetype: mimetype.to_string(),
        }
    }

    pub(crate) fn from_key(key: &str) -> FileObject {
        FileObject {
            key: key.to_string(),
            mimetype: "".to_string(),
        }
    }

    pub(crate) fn from_url(url: &str) -> ServiceResult<FileObject> {
        let url = Url::parse("https://example.com")
            .and_then(|u| u.join(url))
            .map_err(Error::from)?;

        let mimetype = mime_guess::from_path(url.path()).first_or_octet_stream();

        FileObject {
            key: url.path().trim_matches('/').to_string(),
            mimetype: mimetype.to_string(),
        }
        .into()
    }
}

impl Into<ServiceResult<FileObject>> for FileObject {
    fn into(self) -> ServiceResult<FileObject> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_filename_with_extension() {
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let file_obj = FileObject::from_filename("avatar.png", None);
        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert_eq!(file_obj.mimetype, "image/png");
        let parts: Vec<&str> = file_obj.key.split('.').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1], "png");

        let key_parts: Vec<&str> = parts[0].split('-').collect();
        assert_eq!(key_parts[0], "avatar");
        let ts: u64 = key_parts[1].parse().expect("valid timestamp");
        assert!(ts >= before && ts <= after);
    }

    #[test]
    fn test_from_filename_nested_path() {
        let file_obj = FileObject::from_filename("uploads/images/photo.jpg", Some("image/jpeg"));
        assert_eq!(file_obj.mimetype, "image/jpeg");
        assert!(file_obj.key.starts_with("uploads/images/photo-"));
        assert!(file_obj.key.ends_with(".jpg"));
    }

    #[test]
    fn test_from_filename_without_extension() {
        let file_obj = FileObject::from_filename("README", None);
        assert_eq!(file_obj.mimetype, "application/octet-stream");
        assert!(file_obj.key.starts_with("README-"));
        assert!(!file_obj.key.contains('.'));
    }

    #[test]
    fn test_from_filename_multiple_dots() {
        let file_obj = FileObject::from_filename("archive.backup.tar.gz", None);
        assert_eq!(file_obj.mimetype, "application/gzip");
        assert!(file_obj.key.starts_with("archive.backup.tar-"));
        assert!(file_obj.key.ends_with(".gz"));
    }
}
