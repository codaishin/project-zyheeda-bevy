mod behaviors;

pub trait Add<T> {
	fn add(&mut self, value: T);
}
