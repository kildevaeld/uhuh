#[cfg(not(feature = "std"))]
mod nostd {
    use core::fmt::{Debug, Display};
    use core::{convert::Infallible, fmt};

    #[cfg(feature = "alloc")]
    use alloc::{boxed::Box, string::String};

    pub trait Error: Debug + Display {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            None
        }

        #[deprecated(note = "use the Display impl or to_string()")]
        fn description(&self) -> &str {
            "description() is deprecated; use Display"
        }

        #[deprecated(note = "replaced by Error::source, which can support downcasting")]
        #[allow(missing_docs)]
        fn cause(&self) -> Option<&dyn Error> {
            self.source()
        }
    }

    impl Error for fmt::Error {}

    impl Error for Infallible {}

    #[cfg(feature = "alloc")]
    pub type BoxError<'a> = Box<dyn Error + Send + Sync + 'a>;

    #[cfg(feature = "alloc")]
    pub type LocalBoxError<'a> = Box<dyn Error + 'a>;

    #[cfg(feature = "alloc")]
    impl<'a, T> From<T> for Box<dyn Error + 'a>
    where
        T: Error + 'a,
    {
        fn from(value: T) -> Self {
            Box::new(value)
        }
    }

    #[cfg(feature = "alloc")]
    impl<'a, T> From<T> for Box<dyn Error + Send + Sync + 'a>
    where
        T: Error + Send + Sync + 'a,
    {
        fn from(value: T) -> Self {
            Box::new(value)
        }
    }

    #[cfg(feature = "alloc")]
    impl<'a> From<String> for BoxError<'a> {
        fn from(value: String) -> BoxError<'a> {
            struct StringError(String);

            impl fmt::Display for StringError {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str(&self.0)
                }
            }

            impl fmt::Debug for StringError {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.debug_struct("StringError").field("", &self.0).finish()
                }
            }

            impl Error for StringError {}

            Box::new(StringError(value))
        }
    }

    #[cfg(feature = "alloc")]
    impl From<String> for LocalBoxError<'static> {
        fn from(value: String) -> LocalBoxError<'static> {
            let err1: Box<dyn Error + Send + Sync> = From::from(value);
            let err2: Box<dyn Error> = err1;
            err2
        }
    }

    #[cfg(feature = "alloc")]
    impl<'a> From<&str> for BoxError<'a> {
        fn from(value: &str) -> BoxError<'a> {
            From::from(String::from(value))
        }
    }

    #[cfg(feature = "alloc")]
    impl<'a> From<&str> for LocalBoxError<'a> {
        fn from(value: &str) -> LocalBoxError<'a> {
            From::from(String::from(value))
        }
    }

    // impl Error for String {}
}

#[cfg(not(feature = "std"))]
pub use nostd::*;

#[cfg(feature = "std")]
pub use std::error::Error;

#[cfg(feature = "std")]
pub type BoxError<'a> = std::boxed::Box<dyn std::error::Error + Send + Sync + 'a>;

#[cfg(feature = "std")]
pub type LocalBoxError<'a> = std::boxed::Box<dyn std::error::Error + 'a>;
