pub trait Context {
    fn get<T: 'static + Send + Sync>(&self) -> Option<&T>;
}

pub trait ContextBuilder {
    fn register<T: 'static + Send + Sync>(&mut self, value: T) -> Option<T>;
}
