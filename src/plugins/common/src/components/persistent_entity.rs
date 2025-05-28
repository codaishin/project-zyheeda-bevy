use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
#[component(immutable)]
pub struct PersistentEntity(Uuid);

impl Default for PersistentEntity {
	fn default() -> Self {
		Self(Uuid::new_v4())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn two_ids_are_different() {
		let a = PersistentEntity::default();
		let b = PersistentEntity::default();

		assert!(a != b);
	}
}
