use shared::error::ServiceError;

pub type ServiceResult<T> = Result<T, ServiceError>;
