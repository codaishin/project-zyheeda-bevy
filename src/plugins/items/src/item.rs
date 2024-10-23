use bevy::prelude::*;
use common::tools::ModelPath;
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Debug, PartialEq, Clone)]
pub struct Item<TContent> {
	pub name: &'static str,
	pub model: Option<ModelPath>,
	pub content: TContent,
}

impl<TContent> Default for Item<TContent>
where
	TContent: Default,
{
	fn default() -> Self {
		Self {
			name: default(),
			model: default(),
			content: default(),
		}
	}
}

impl<TContent> Display for Item<TContent> {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "Item({})", self.name)
	}
}
