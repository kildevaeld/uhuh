use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("request error: {0}")]
    Request(#[from] RequestError),
    #[error("task error: {0}")]
    Task(#[from] TaskError),
}

#[derive(Debug, ThisError)]
#[error("{0}")]
pub struct RequestError(Box<dyn std::error::Error + Send + Sync>);

impl RequestError {
    pub fn new<E: std::error::Error + Send + Sync + 'static>(error: E) -> RequestError {
        RequestError(Box::new(error))
    }
}

#[derive(Debug, ThisError)]
#[error("{0}")]
pub struct TaskError(Box<dyn std::error::Error + Send + Sync>);

impl TaskError {
    pub fn new<E: std::error::Error + Send + Sync + 'static>(error: E) -> TaskError {
        TaskError(Box::new(error))
    }
}
