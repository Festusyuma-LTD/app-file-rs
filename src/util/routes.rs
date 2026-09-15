#[derive(Default)]
pub struct DisabledRoutes {
    pub init_upload: bool,
    pub url: bool,
}

pub enum Route {
    InitUpload,
    Url,
}

#[derive(Default)]
pub struct DisabledRoutesBuilder {
    init_upload: Option<bool>,
    url: Option<bool>,
}

impl DisabledRoutesBuilder {
    pub fn disable(mut self, route: Route) -> Self {
        match route {
            Route::InitUpload => self.init_upload = Some(true),
            Route::Url => self.url = Some(true),
        }

        self
    }
}

impl DisabledRoutes {
    pub fn builder() -> DisabledRoutesBuilder {
        DisabledRoutesBuilder::default()
    }

    pub fn new(builder: DisabledRoutesBuilder) -> Self {
        Self {
            init_upload: builder.init_upload.unwrap_or_default(),
            url: builder.url.unwrap_or_default(),
        }
    }

    pub(crate) fn is_disabled(&self, path: &str) -> bool {
        match path.strip_prefix("/upload/").unwrap_or(path) {
            "init_upload" => self.init_upload,
            "url" => self.url,
            _ => false,
        }
    }
}
