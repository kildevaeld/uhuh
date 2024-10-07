pub use core::error::Error;

#[cfg(feature = "alloc")]
pub type BoxError<'a> = alloc::boxed::Box<dyn Error + Send + Sync + 'a>;

#[cfg(feature = "alloc")]
pub type LocalBoxError<'a> = alloc::boxed::Box<dyn Error + 'a>;
