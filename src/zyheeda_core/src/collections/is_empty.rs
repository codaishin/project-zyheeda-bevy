mod array;
mod slice;
mod vec;

pub trait IsEmpty {
	fn is_empty(&self) -> bool;
}
