use std::{
	error::Error,
	fmt::{Debug, Display},
};

#[derive(Debug, PartialEq)]
pub struct NotInRange<T> {
	pub lower_limit: Limit<T>,
	pub upper_limit: Limit<T>,
	pub value: T,
}

#[derive(Debug, PartialEq)]
pub enum Limit<T> {
	Inclusive(T),
	Exclusive(T),
}

impl<T> Display for NotInRange<T>
where
	T: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Value `{:?}` is out of bounds. Expected to be within `{:?}` and `{:?}`.",
			self.value, self.lower_limit, self.upper_limit,
		)
	}
}

impl<T> Error for NotInRange<T> where T: Debug {}
