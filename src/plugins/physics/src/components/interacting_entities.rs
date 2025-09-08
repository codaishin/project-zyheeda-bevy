use bevy::prelude::{Component, Entity};
use std::collections::{HashSet, hash_set::Iter};

#[derive(Component, Default, Debug, PartialEq, Clone)]
pub struct InteractingEntities(pub(crate) HashSet<Entity>);

impl InteractingEntities {
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(entities: [Entity; N]) -> Self {
		Self(HashSet::from(entities))
	}

	pub(crate) fn contains(&self, entity: &Entity) -> bool {
		self.0.contains(entity)
	}

	pub(crate) fn iter(&self) -> Iter<'_, Entity> {
		self.0.iter()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn contains() {
		let entities = InteractingEntities::new([Entity::from_raw(1), Entity::from_raw(2)]);

		assert_eq!(
			[true, false],
			[
				entities.contains(&Entity::from_raw(1)),
				entities.contains(&Entity::from_raw(3))
			]
		);
	}
}
