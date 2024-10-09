use alloc::{
    fmt,
    string::{String, ToString},
};
pub use daserror::{BoxError, Error};

pub trait ResultContext: Sized {
    fn with_context(self, context: impl ToString) -> Self;
}

impl<T> ResultContext for Result<T, UhuhError> {
    fn with_context(self, context: impl ToString) -> Self {
        match self {
            Ok(ret) => Ok(ret),
            Err(err) => Err(err.with_context(context)),
        }
    }
}

#[derive(Debug)]
pub struct UhuhError {
    inner: BoxError<'static>,
    context: Option<String>,
}

impl UhuhError {
    pub fn new<T>(error: T) -> UhuhError
    where
        T: Into<BoxError<'static>>,
    {
        UhuhError {
            inner: error.into(),
            context: None,
        }
    }

    pub fn with_context(mut self, context: impl ToString) -> Self {
        if self.context.is_some() {
            UhuhError::new(self).with_context(context)
        } else {
            self.context = Some(context.to_string());
            self
        }
    }
}

impl fmt::Display for UhuhError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(context) = self.context.as_ref() {
            write!(f, "{context}: ")?;
        }
        write!(f, "{}", self.inner)
    }
}

impl Error for UhuhError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.inner.source()
    }
}

impl From<BoxError<'static>> for UhuhError {
    fn from(value: BoxError<'static>) -> Self {
        UhuhError::new(value)
    }
}
