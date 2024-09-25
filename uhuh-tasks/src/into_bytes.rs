use async_trait::async_trait;
use bytes::Bytes;
use core::convert::Infallible;

#[async_trait]
pub trait IntoBytes: Sized {
    type Error;
    async fn into_bytes(self) -> Result<Bytes, Self::Error>;
}

#[async_trait]
impl IntoBytes for Bytes {
    type Error = Infallible;
    async fn into_bytes(self) -> Result<Bytes, Self::Error> {
        Ok(self)
    }
}

#[async_trait]
impl IntoBytes for Vec<u8> {
    type Error = Infallible;
    async fn into_bytes(self) -> Result<Bytes, Self::Error> {
        Ok(self.into())
    }
}

#[async_trait]
impl<'a> IntoBytes for &'a [u8] {
    type Error = Infallible;
    async fn into_bytes(self) -> Result<Bytes, Self::Error> {
        Ok(self.to_vec().into())
    }
}
