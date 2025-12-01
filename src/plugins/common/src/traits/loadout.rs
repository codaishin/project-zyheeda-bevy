use crate::tools::action_key::slot::SlotKey;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref, sync::Arc};

pub trait LoadoutConfig {
	fn inventory(&self) -> impl Iterator<Item = Option<ItemName>>;
	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<ItemName>)>;
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ItemName(Arc<str>);

impl From<String> for ItemName {
	fn from(name: String) -> Self {
		Self(Arc::from(name))
	}
}

impl From<&str> for ItemName {
	fn from(name: &str) -> Self {
		Self(Arc::from(name))
	}
}

impl Deref for ItemName {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Display for ItemName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}
