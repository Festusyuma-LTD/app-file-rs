use crate::UploadService;
use crate::util::routes::DisabledRoutes;
use axum::extract::State;
use std::sync::Arc;

pub struct ServiceState {
    pub upload_service: UploadService,
    pub disabled_routes: DisabledRoutes,
}

pub type ServiceStateType = State<Arc<ServiceState>>;
