mod behaviors;

pub trait GetMut<T> {
	fn get(&mut self) -> Option<&mut T>;
}

pub trait Get<T> {
	fn get(&self) -> Option<T>;
}
