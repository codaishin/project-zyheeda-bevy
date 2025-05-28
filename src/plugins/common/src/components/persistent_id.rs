use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
#[component(immutable)]
pub struct PersistentId(Uuid);

impl Default for PersistentId {
	fn default() -> Self {
		Self(Uuid::new_v4())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn two_ids_are_different() {
		let a = PersistentId::default();
		let b = PersistentId::default();

		assert!(a != b);
	}
}
