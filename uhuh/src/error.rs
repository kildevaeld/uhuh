use core::fmt;

#[derive(Debug)]
pub struct Error {
    inner: Box<dyn std::error::Error + Send + Sync>,
}

impl Error {
    pub fn new<T>(error: T) -> Error
    where
        T: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self {
            inner: error.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.inner)
    }
}

impl From<vaerdi::ser::SerializerError> for Error {
    fn from(value: vaerdi::ser::SerializerError) -> Self {
        Self::new(value)
    }
}

impl From<johnfig::Error> for Error {
    fn from(value: johnfig::Error) -> Self {
        Self::new(value)
    }
}
