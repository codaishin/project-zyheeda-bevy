use bevy::prelude::{Component, Entity};
use std::collections::{HashSet, hash_set::Iter};

#[derive(Component, Default, Debug, PartialEq, Clone)]
pub struct InteractingEntities(pub(crate) HashSet<Entity>);

impl InteractingEntities {
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(entities: [Entity; N]) -> Self {
		Self(HashSet::from(entities))
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn contains(&self, entity: &Entity) -> bool {
		self.0.contains(entity)
	}

	pub fn iter(&self) -> Iter<'_, Entity> {
		self.0.iter()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn len() {
		let entities = InteractingEntities::new([Entity::from_raw(1), Entity::from_raw(2)]);

		assert_eq!(2, entities.len());
	}

	#[test]
	fn is_empty() {
		let not_empty = InteractingEntities::new([Entity::from_raw(1), Entity::from_raw(2)]);
		let empty = InteractingEntities::new([]);

		assert_eq!([false, true], [not_empty.is_empty(), empty.is_empty()]);
	}

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
