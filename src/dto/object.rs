use crate::util::error::ServiceResult;

use anyhow::Error;
use serde::Serialize;
use std::path::Path;
use std::str::FromStr;
use url::Url;

#[derive(Serialize)]
pub struct FileObject {
    pub key: String,
    pub mimetype: String,
    pub url: Option<String>,
}

impl FileObject {
    pub(crate) fn new(key: String, mimetype: String, url: Option<String>) -> FileObject {
        FileObject { key, mimetype, url }
    }

    pub(crate) fn from_filename(file_name: &str, mimetype: Option<&str>) -> FileObject {
        let path = Path::new(file_name);

        let mimetype = mimetype
            .map(|v| mime_guess::Mime::from_str(v).ok())
            .unwrap_or_else(|| mime_guess::from_path(file_name).first())
            .unwrap_or(mime_guess::mime::APPLICATION_OCTET_STREAM);

        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name);

        let ext = [path.extension().and_then(|s| s.to_str()).unwrap_or("")];
        let ext = mime_guess::get_mime_extensions(&mimetype).unwrap_or(&ext);

        let mut file_name_tre = [file_name].to_vec();
        file_name_tre.extend(ext);

        FileObject {
            key: file_name_tre.join("."),
            mimetype: mimetype.to_string(),
            url: None,
        }
    }

    pub(crate) fn from_key(key: &str) -> FileObject {
        FileObject {
            key: key.to_string(),
            mimetype: "".to_string(),
            url: None,
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
            url: Some(url.to_string()),
        }
        .into()
    }
}

impl Into<ServiceResult<FileObject>> for FileObject {
    fn into(self) -> ServiceResult<FileObject> {
        Ok(self)
    }
}
