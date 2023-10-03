mod behaviors;

pub trait Get<T> {
	fn get(&mut self) -> Option<&mut T>;
}
