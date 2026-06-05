use bevy::prelude::*;
use std::{
	collections::{HashMap, HashSet, hash_map::Iter},
	marker::PhantomData,
	sync::LazyLock,
};

static EMPTY: LazyLock<HashSet<Entity>> = LazyLock::new(HashSet::default);

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct RootCollisions<T> {
	ongoing: HashMap<Entity, HashSet<Entity>>,
	old: HashMap<Entity, HashSet<Entity>>,
	_p: PhantomData<T>,
}

impl<T> RootCollisions<T> {
	pub(crate) fn update<TCollisions>(&mut self, entity: Entity, collisions: TCollisions)
	where
		TCollisions: IntoIterator<Item = Entity>,
	{
		let ongoing = self.ongoing.entry(entity).or_default();

		ongoing.extend(collisions);
	}

	pub(crate) fn rotate(&mut self) {
		std::mem::swap(&mut self.ongoing, &mut self.old);

		self.ongoing.clear();
	}

	pub(crate) fn ongoing(&self, entity: &Entity) -> &'_ HashSet<Entity> {
		self.ongoing.get(entity).unwrap_or(&*EMPTY)
	}

	pub(crate) fn just_stopped(&self, entity: &Entity) -> HashSet<Entity> {
		let ongoing = self.ongoing.get(entity).unwrap_or(&*EMPTY);
		let old = self.old.get(entity).unwrap_or(&*EMPTY);

		HashSet::from_iter(old.iter().filter(|old| !ongoing.contains(old)).copied())
	}

	pub(crate) fn changed(&self, entity: &Entity) -> bool {
		self.old.get(entity) != self.ongoing.get(entity)
	}
}

impl<T> Default for RootCollisions<T> {
	fn default() -> Self {
		Self {
			ongoing: HashMap::default(),
			old: HashMap::default(),
			_p: PhantomData,
		}
	}
}

impl<T, TFrom> From<TFrom> for RootCollisions<T>
where
	TFrom: Into<HashMap<Entity, HashSet<Entity>>>,
{
	fn from(ongoing: TFrom) -> Self {
		let ongoing = ongoing.into();

		Self {
			ongoing,
			..default()
		}
	}
}

impl<'a, T> IntoIterator for &'a RootCollisions<T> {
	type Item = (&'a Entity, &'a HashSet<Entity>);
	type IntoIter = Iter<'a, Entity, HashSet<Entity>>;

	fn into_iter(self) -> Self::IntoIter {
		self.ongoing.iter()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::fake_entity;

	#[test]
	fn add_interactions() {
		let mut interactions = RootCollisions::<()>::default();

		interactions.update(fake_entity!(1), [fake_entity!(2)]);

		assert_eq!(
			(&HashSet::from([fake_entity!(2)]), true),
			(
				interactions.ongoing(&fake_entity!(1)),
				interactions.changed(&fake_entity!(1))
			)
		);
	}

	#[test]
	fn clear() {
		let mut interactions = RootCollisions::<()>::default();

		interactions.update(fake_entity!(1), [fake_entity!(2)]);
		interactions.rotate();

		assert_eq!(
			(&HashSet::from([]), true, false),
			(
				interactions.ongoing(&fake_entity!(1)),
				interactions.changed(&fake_entity!(1)),
				interactions.changed(&fake_entity!(3)),
			)
		);
	}

	#[test]
	fn after_rotate_unchanged() {
		let mut interactions = RootCollisions::<()>::default();

		interactions.update(fake_entity!(1), [fake_entity!(2)]);
		interactions.rotate();
		interactions.update(fake_entity!(1), [fake_entity!(2)]);

		assert_eq!(
			(&HashSet::from([fake_entity!(2)]), false),
			(
				interactions.ongoing(&fake_entity!(1)),
				interactions.changed(&fake_entity!(1))
			)
		);
	}

	#[test]
	fn after_rotate_changed() {
		let mut interactions = RootCollisions::<()>::default();

		interactions.update(fake_entity!(1), [fake_entity!(2)]);
		interactions.rotate();
		interactions.update(fake_entity!(1), [fake_entity!(3)]);

		assert_eq!(
			(&HashSet::from([fake_entity!(3)]), true),
			(
				interactions.ongoing(&fake_entity!(1)),
				interactions.changed(&fake_entity!(1))
			)
		);
	}

	#[test]
	fn after_rotate_changed_if_entities_of_old_missing() {
		let mut interactions = RootCollisions::<()>::default();

		interactions.update(fake_entity!(1), [fake_entity!(2), fake_entity!(3)]);
		interactions.rotate();
		interactions.update(fake_entity!(1), [fake_entity!(2)]);

		assert_eq!(
			(&HashSet::from([fake_entity!(2)]), true),
			(
				interactions.ongoing(&fake_entity!(1)),
				interactions.changed(&fake_entity!(1))
			)
		);
	}

	#[test]
	fn after_rotate_unchanged_if_entities_old_and_ongoing_match() {
		let mut interactions = RootCollisions::<()>::default();

		interactions.update(fake_entity!(1), [fake_entity!(2), fake_entity!(3)]);
		interactions.rotate();
		interactions.update(fake_entity!(1), [fake_entity!(2)]);
		interactions.update(fake_entity!(1), [fake_entity!(3)]);

		assert_eq!(
			(&HashSet::from([fake_entity!(2), fake_entity!(3)]), false),
			(
				interactions.ongoing(&fake_entity!(1)),
				interactions.changed(&fake_entity!(1))
			)
		);
	}

	#[test]
	fn iterate_just_stopped() {
		let mut interactions = RootCollisions::<()>::default();

		interactions.update(
			fake_entity!(1),
			[
				fake_entity!(2),
				fake_entity!(3),
				fake_entity!(4),
				fake_entity!(5),
				fake_entity!(6),
			],
		);
		interactions.rotate();
		interactions.update(
			fake_entity!(1),
			[
				fake_entity!(2),
				fake_entity!(3),
				fake_entity!(5),
				fake_entity!(6),
			],
		);

		assert_eq!(
			HashSet::from([fake_entity!(4)]),
			interactions.just_stopped(&fake_entity!(1)),
		);
	}
}
