use std::{
	error::Error,
	fmt::{Debug, Display},
};

#[derive(Debug, PartialEq)]
pub struct NotInBetween<T> {
	pub lower_limit: T,
	pub upper_limit: T,
	pub value: T,
}

impl<T> Display for NotInBetween<T>
where
	T: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Value `{:?}` is out of bounds. Expected to be greater than `{:?}` and lesser than `{:?}`.",
			self.value, self.lower_limit, self.upper_limit,
		)
	}
}

impl<T> Error for NotInBetween<T> where T: Debug {}
