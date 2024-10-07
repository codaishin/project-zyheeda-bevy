mod removed_components;

pub trait Read<'a> {
	type TReturn;

	fn read(&'a mut self) -> Self::TReturn;
}
