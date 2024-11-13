pub mod dto;

use bevy::{asset::Asset, reflect::TypePath};
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Default, Debug, PartialEq, Clone, Asset, TypePath)]
pub struct Item<TContent>
where
	TContent: TypePath + Sync + Send + 'static,
{
	pub name: String,
	pub content: TContent,
}

impl<TContent> Item<TContent>
where
	TContent: Default + TypePath + Sync + Send + 'static,
{
	pub fn named<TName>(name: TName) -> Self
	where
		String: From<TName>,
	{
		Self {
			name: String::from(name),
			content: TContent::default(),
		}
	}

	pub fn with_content(mut self, content: TContent) -> Self {
		self.content = content;
		self
	}
}

impl<TContent> Display for Item<TContent>
where
	TContent: TypePath + Sync + Send + 'static,
{
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "Item({})", self.name)
	}
}
