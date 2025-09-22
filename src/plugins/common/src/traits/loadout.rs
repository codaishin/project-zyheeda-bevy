use crate::tools::action_key::slot::SlotKey;
use serde::{Deserialize, Serialize};

pub trait LoadoutConfig {
	fn inventory(&self) -> impl Iterator<Item = Option<ItemName>>;
	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<ItemName>)>;
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ItemName(pub String);

impl<T> From<T> for ItemName
where
	T: Into<String>,
{
	fn from(name: T) -> Self {
		Self(name.into())
	}
}
