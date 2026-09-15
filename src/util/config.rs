use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, Region};
use std::sync::Arc;

pub struct CdnKey {
    pub key_id: String,
    pub private_key_path: String,
}

pub struct Config {
    pub client: aws_sdk_s3::Client,
    pub bucket: String,
    pub base_url: String,
    pub cdn_key: Option<CdnKey>,
}

impl Config {
    async fn new(
        region: String,
        bucket: String,
        base_url: String,
        cdn_key: Option<CdnKey>,
    ) -> Arc<Config> {
        let region_provider = RegionProviderChain::default_provider().or_else(Region::new(region));
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;

        Arc::new(Config {
            bucket,
            base_url,
            cdn_key,
            client: aws_sdk_s3::Client::new(&config),
        })
    }
}

pub struct ConfigBuilder {
    region: Option<String>,
    bucket: Option<String>,
    base_url: Option<String>,
    cdn_key: Option<CdnKey>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            region: None,
            bucket: None,
            base_url: None,
            cdn_key: None,
        }
    }

    pub fn region(mut self, region: &str) -> Self {
        self.region = Some(region.to_string());
        self
    }

    pub fn cdn_key(mut self, key_id: &str, private_key: &str) -> Self {
        self.cdn_key = Some(CdnKey {
            key_id: key_id.to_string(),
            private_key_path: private_key.to_string(),
        });

        self
    }

    pub fn bucket(mut self, bucket: &str) -> Self {
        self.bucket = Some(bucket.to_string());
        self
    }

    pub fn base_url(mut self, base_url: &str) -> Self {
        self.base_url = Some(base_url.trim_end_matches("/").to_string());
        self
    }

    pub fn build(self) -> impl Future<Output = Arc<Config>> {
        let region = self.region.map_or_else(|| String::new(), |r| r);
        let bucket = self.bucket.map_or_else(|| String::new(), |b| b);
        let base_url = self.base_url.map_or_else(|| String::new(), |b| b);

        assert!(!region.is_empty(), "Region is required");
        assert!(!bucket.is_empty(), "Bucket is required");
        assert!(!base_url.is_empty(), "Base URL is required");

        Config::new(region, bucket, base_url, self.cdn_key)
    }
}
