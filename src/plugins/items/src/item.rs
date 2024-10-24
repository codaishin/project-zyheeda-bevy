use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Item<TContent> {
	pub name: &'static str,
	pub content: TContent,
}

impl<TContent> Display for Item<TContent> {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "Item({})", self.name)
	}
}
