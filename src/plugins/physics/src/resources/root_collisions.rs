use crate::components::interactions_changed::InteractionsChanged;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{accessors::get::TryApplyOn, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaCommands,
};
use std::{
	collections::{HashMap, HashSet, hash_map::Iter},
	marker::PhantomData,
	sync::LazyLock,
};

static EMPTY: LazyLock<HashSet<Entity>> = LazyLock::new(HashSet::default);

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct RootCollisions<T> {
	ongoing: HashMap<Entity, HashSet<Entity>>,
	buffer: HashMap<Entity, HashSet<Entity>>,
	_p: PhantomData<T>,
}

impl<T> RootCollisions<T> {
	#[cfg(test)]
	pub(crate) fn from_ongoing<TFrom>(ongoing: TFrom) -> Self
	where
		TFrom: Into<HashMap<Entity, HashSet<Entity>>>,
	{
		let ongoing = ongoing.into();

		Self {
			ongoing,
			..default()
		}
	}

	#[cfg(test)]
	pub(crate) fn from_buffer<TFrom>(buffer: TFrom) -> Self
	where
		TFrom: Into<HashMap<Entity, HashSet<Entity>>>,
	{
		let buffer = buffer.into();

		Self {
			buffer,
			..default()
		}
	}

	pub(crate) fn ongoing(&self, entity: &Entity) -> &'_ HashSet<Entity> {
		self.ongoing.get(entity).unwrap_or(&*EMPTY)
	}

	#[cfg(test)]
	pub(crate) fn buffer(&self, entity: &Entity) -> &'_ HashSet<Entity> {
		self.buffer.get(entity).unwrap_or(&*EMPTY)
	}

	fn update_buffer<TCollisions>(
		&mut self,
		entity: Entity,
		collisions: TCollisions,
	) -> NewCollisions
	where
		TCollisions: IntoIterator<Item = Entity>,
	{
		let ongoing = self.ongoing.get(&entity).unwrap_or(&*EMPTY);
		let buffer = self.buffer.entry(entity).or_default();
		let mut added_collisions = false;

		for collision in collisions {
			if !ongoing.contains(&collision) {
				added_collisions = true;
			}
			buffer.insert(collision);
		}

		NewCollisions(added_collisions)
	}

	fn rotate(&mut self) -> RemovedCollisions {
		let mut removed_collisions = HashSet::from([]);
		for (entity, ongoing) in &self.ongoing {
			let buffer = self.buffer.get(entity).unwrap_or(&*EMPTY);
			if buffer == ongoing {
				continue;
			}

			removed_collisions.insert(*entity);
		}

		std::mem::swap(&mut self.ongoing, &mut self.buffer);

		self.buffer.clear();

		RemovedCollisions(removed_collisions.into_iter())
	}
}

impl<T> Default for RootCollisions<T> {
	fn default() -> Self {
		Self {
			ongoing: HashMap::default(),
			buffer: HashMap::default(),
			_p: PhantomData,
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

#[derive(SystemParam)]
pub(crate) struct RootCollisionsParam<'w, 's, T>
where
	T: ThreadSafe,
{
	root_collisions: ResMut<'w, RootCollisions<T>>,
	commands: ZyheedaCommands<'w, 's>,
}

impl<'w, 's, T> RootCollisionsParam<'w, 's, T>
where
	T: ThreadSafe,
{
	pub(crate) fn update<TCollisions>(&mut self, entity: Entity, collisions: TCollisions)
	where
		TCollisions: IntoIterator<Item = Entity>,
	{
		if self.root_collisions.update_buffer(entity, collisions) == NewCollisions(true) {
			self.commands.try_apply_on(&entity, |mut e| {
				e.try_insert(InteractionsChanged);
			});
		}
	}

	pub(crate) fn rotate(mut p: RootCollisionsParam<T>) {
		for removed_collisions in p.root_collisions.rotate() {
			p.commands.try_apply_on(&removed_collisions, |mut e| {
				e.try_insert(InteractionsChanged);
			});
		}
	}
}

#[derive(PartialEq)]
struct NewCollisions(bool);

struct RemovedCollisions(std::collections::hash_set::IntoIter<Entity>);

impl Iterator for RemovedCollisions {
	type Item = Entity;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::interactions_changed::InteractionsChanged;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::{SingleThreadedApp, fake_entity};

	fn setup(root_collisions: RootCollisions<()>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(root_collisions);

		app
	}

	#[test]
	fn add_interactions_to_new() -> Result<(), RunSystemError> {
		let mut app = setup(RootCollisions::<()>::default());
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: RootCollisionsParam<()>| {
				p.update(entity, [fake_entity!(2)]);
			})?;

		assert_eq!(
			(&HashSet::from([fake_entity!(2)]), &HashSet::from([])),
			(
				app.world().resource::<RootCollisions<()>>().buffer(&entity),
				app.world()
					.resource::<RootCollisions<()>>()
					.ongoing(&entity),
			)
		);
		Ok(())
	}

	#[test]
	fn add_interactions_when_already_present() -> Result<(), RunSystemError> {
		let mut app = setup(RootCollisions::<()>::default());
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut()
			.insert_resource(RootCollisions::<()>::from_ongoing([(
				entity,
				HashSet::from([fake_entity!(2)]),
			)]));

		app.world_mut()
			.run_system_once(move |mut p: RootCollisionsParam<()>| {
				p.update(entity, [fake_entity!(2)]);
			})?;

		assert_eq!(
			(
				&HashSet::from([fake_entity!(2)]),
				&HashSet::from([fake_entity!(2)]),
			),
			(
				app.world().resource::<RootCollisions<()>>().buffer(&entity),
				app.world()
					.resource::<RootCollisions<()>>()
					.ongoing(&entity),
			)
		);
		Ok(())
	}

	#[test]
	fn mark_new_as_changed() -> Result<(), RunSystemError> {
		let mut app = setup(RootCollisions::<()>::default());
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: RootCollisionsParam<()>| {
				p.update(entity, [fake_entity!(2)]);
			})?;

		assert!(app.world().entity(entity).contains::<InteractionsChanged>());
		Ok(())
	}

	#[test]
	fn mark_new_as_changed_among_several_unchanged() -> Result<(), RunSystemError> {
		let mut app = setup(RootCollisions::<()>::default());
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut().insert_resource(RootCollisions::<()> {
			ongoing: HashMap::from([(entity, HashSet::from([fake_entity!(1), fake_entity!(3)]))]),
			..default()
		});

		app.world_mut()
			.run_system_once(move |mut p: RootCollisionsParam<()>| {
				p.update(entity, [fake_entity!(2), fake_entity!(1)]);
			})?;

		assert!(app.world().entity(entity).contains::<InteractionsChanged>());
		Ok(())
	}

	#[test]
	fn do_not_mark_non_new_as_changed() -> Result<(), RunSystemError> {
		let mut app = setup(RootCollisions::<()>::default());
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut().insert_resource(RootCollisions::<()> {
			ongoing: HashMap::from([(entity, HashSet::from([fake_entity!(2)]))]),
			..default()
		});

		app.world_mut()
			.run_system_once(move |mut p: RootCollisionsParam<()>| {
				p.update(entity, [fake_entity!(2)]);
			})?;

		assert!(!app.world().entity(entity).contains::<InteractionsChanged>());
		Ok(())
	}

	#[test]
	fn mark_non_new_as_changed_when_interacting_with_new() -> Result<(), RunSystemError> {
		let mut app = setup(RootCollisions::<()>::default());
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut()
			.insert_resource(RootCollisions::<()>::from_ongoing([(
				entity,
				HashSet::from([fake_entity!(2)]),
			)]));

		app.world_mut()
			.run_system_once(move |mut p: RootCollisionsParam<()>| {
				p.update(entity, [fake_entity!(3)]);
			})?;

		assert!(app.world().entity(entity).contains::<InteractionsChanged>());
		Ok(())
	}

	#[test]
	fn rotate() -> Result<(), RunSystemError> {
		let mut app = setup(RootCollisions::<()>::default());
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut().insert_resource(RootCollisions::<()> {
			buffer: HashMap::from([(entity, HashSet::from([fake_entity!(2)]))]),
			ongoing: HashMap::from([(entity, HashSet::from([fake_entity!(1)]))]),
			..default()
		});

		app.world_mut()
			.run_system_once(RootCollisionsParam::<()>::rotate)?;

		assert_eq!(
			(&HashSet::from([]), &HashSet::from([fake_entity!(2)])),
			(
				app.world().resource::<RootCollisions<()>>().buffer(&entity),
				app.world()
					.resource::<RootCollisions<()>>()
					.ongoing(&entity),
			)
		);
		Ok(())
	}

	#[test]
	fn mark_changed_on_rotate() -> Result<(), RunSystemError> {
		let mut app = setup(RootCollisions::<()>::default());
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut().insert_resource(RootCollisions::<()> {
			buffer: HashMap::from([(entity, HashSet::from([fake_entity!(2)]))]),
			ongoing: HashMap::from([(entity, HashSet::from([fake_entity!(1)]))]),
			..default()
		});

		app.world_mut()
			.run_system_once(RootCollisionsParam::<()>::rotate)?;

		assert!(app.world().entity(entity).contains::<InteractionsChanged>());
		Ok(())
	}

	#[test]
	fn do_not_mark_changed_when_interactions_match() -> Result<(), RunSystemError> {
		let mut app = setup(RootCollisions::<()>::default());
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut().insert_resource(RootCollisions::<()> {
			buffer: HashMap::from([(entity, HashSet::from([fake_entity!(1)]))]),
			ongoing: HashMap::from([(entity, HashSet::from([fake_entity!(1)]))]),
			..default()
		});

		app.world_mut()
			.run_system_once(RootCollisionsParam::<()>::rotate)?;

		assert!(!app.world().entity(entity).contains::<InteractionsChanged>());
		Ok(())
	}

	#[test]
	fn mark_changed_when_interactions_removed() -> Result<(), RunSystemError> {
		let mut app = setup(RootCollisions::<()>::default());
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut()
			.insert_resource(RootCollisions::<()>::from_ongoing([(
				entity,
				HashSet::from([fake_entity!(1)]),
			)]));

		app.world_mut()
			.run_system_once(RootCollisionsParam::<()>::rotate)?;

		assert!(app.world().entity(entity).contains::<InteractionsChanged>());
		Ok(())
	}
}
