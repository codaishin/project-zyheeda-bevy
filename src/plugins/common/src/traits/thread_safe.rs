pub trait ThreadSafe: Sync + Send + 'static {}

impl<T> ThreadSafe for T where T: Sync + Send + 'static {}
