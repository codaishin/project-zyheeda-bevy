use bevy::prelude::{Component, Entity};
use common::components::collider_relations::ChildColliderOf;
use std::collections::HashSet;

#[derive(Component, Default, Debug, PartialEq, Clone)]
pub struct InteractingEntities(pub(crate) HashSet<ChildColliderOf>);

impl InteractingEntities {
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(entities: [ChildColliderOf; N]) -> Self {
		Self(HashSet::from(entities))
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn contains(&self, entity: &Entity) -> bool {
		self.0.contains(&ChildColliderOf(*entity))
	}

	pub fn iter(&self) -> impl Iterator<Item = &Entity> {
		self.0.iter().map(|ChildColliderOf(r)| r)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn len() {
		let entities = InteractingEntities::new([
			ChildColliderOf(Entity::from_raw(1)),
			ChildColliderOf(Entity::from_raw(2)),
		]);

		assert_eq!(2, entities.len());
	}

	#[test]
	fn is_empty() {
		let not_empty = InteractingEntities::new([
			ChildColliderOf(Entity::from_raw(1)),
			ChildColliderOf(Entity::from_raw(2)),
		]);
		let empty = InteractingEntities::new([]);

		assert_eq!([false, true], [not_empty.is_empty(), empty.is_empty()]);
	}

	#[test]
	fn contains() {
		let entities = InteractingEntities::new([
			ChildColliderOf(Entity::from_raw(1)),
			ChildColliderOf(Entity::from_raw(2)),
		]);

		assert_eq!(
			[true, false],
			[
				entities.contains(&Entity::from_raw(1)),
				entities.contains(&Entity::from_raw(3))
			]
		);
	}
}
