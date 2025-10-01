use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IsNot<T>(PhantomData<T>);

impl<T> IsNot<T> {
	pub fn target_type() -> Self {
		Self(PhantomData)
	}
}
