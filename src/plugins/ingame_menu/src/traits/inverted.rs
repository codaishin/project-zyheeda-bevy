pub mod menu_state;

pub trait Inverted<T> {
	fn inverted(&self) -> Self;
}
