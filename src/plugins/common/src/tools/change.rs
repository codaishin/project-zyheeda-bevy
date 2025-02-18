use std::fmt::Display;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum Change<T> {
	#[default]
	None,
	Some(T),
}

impl<T> Change<T> {
	pub fn expect<TMessage>(self, msg: TMessage) -> T
	where
		TMessage: Display,
	{
		match self {
			Change::None => panic!("{msg}"),
			Change::Some(value) => value,
		}
	}

	pub fn ok(self) -> Option<T> {
		match self {
			Change::None => None,
			Change::Some(value) => Some(value),
		}
	}

	pub fn is_some(&self) -> bool {
		match self {
			Change::None => false,
			Change::Some(_) => true,
		}
	}

	pub fn is_none(&self) -> bool {
		match self {
			Change::None => true,
			Change::Some(_) => false,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn expect_some() {
		let change = Change::Some(42);

		assert_eq!(42, change.expect("ERROR"));
	}

	#[test]
	#[should_panic(expected = "ERROR")]
	fn expect_none() {
		let change = Change::None as Change<u32>;

		change.expect("ERROR");
	}

	#[test]
	fn ok_some() {
		let change = Change::Some(42);

		assert_eq!(Some(42), change.ok());
	}

	#[test]
	fn ok_none() {
		let change = Change::None as Change<u32>;

		assert_eq!(None, change.ok());
	}

	#[test]
	fn is_some_true() {
		let change = Change::Some(42);

		assert!(change.is_some());
	}

	#[test]
	fn is_some_false() {
		let change = Change::None as Change<u32>;

		assert!(!change.is_some());
	}

	#[test]
	fn is_none_true() {
		let change = Change::None as Change<u32>;

		assert!(change.is_none());
	}

	#[test]
	fn is_none_false() {
		let change = Change::Some(42);

		assert!(!change.is_none());
	}
}
