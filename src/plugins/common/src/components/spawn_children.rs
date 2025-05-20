use bevy::prelude::*;

/// Defines children that should be added to an [`Entity`]
///
/// This is a command like component and will be removed from
/// the [`Entity`] after the corresponding children have been added.
#[derive(Component, Debug, PartialEq)]
pub struct SpawnChildren(pub fn(&mut ChildSpawnerCommands));

impl SpawnChildren {
	pub(crate) fn system(mut commands: Commands, spawners: Query<(Entity, &Self)>) {
		for (entity, spawner) in &spawners {
			let Ok(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			let SpawnChildren(spawn_fn) = spawner;

			entity.with_children(spawn_fn);
			entity.remove::<Self>();
		}
	}
}

/// Defines children that should be added to an [`Entity`]
///
/// It can be used to derive children from data on a parent component.
///
/// This is a command like component and will be removed from
/// the [`Entity`] after the corresponding children have been added.
///
/// <div class="warning">
///   Only works, if [`Self::system`] has been registered
/// </div>
#[derive(Component, Debug, PartialEq)]
pub struct SpawnChildrenFromParent<TParent>(pub fn(&mut ChildSpawnerCommands, &TParent))
where
	TParent: Component;

impl<TParent> SpawnChildrenFromParent<TParent>
where
	TParent: Component,
{
	pub fn system(mut commands: Commands, spawners: Query<(Entity, &Self, &TParent)>) {
		for (entity, spawner, parent) in &spawners {
			let Ok(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			let SpawnChildrenFromParent(spawn_fn) = spawner;

			entity.with_children(|entity| spawn_fn(entity, parent));
			entity.remove::<Self>();
		}
	}
}

#[cfg(test)]
mod test_spawn_children {
	use super::*;
	use crate::{assert_count, get_children, test_tools::utils::SingleThreadedApp};

	#[derive(Component, Debug, PartialEq)]
	struct _Component;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, SpawnChildren::system);

		app
	}

	#[test]
	fn spawn_children() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(SpawnChildren(|parent| {
				parent.spawn(_Component);
			}))
			.id();

		app.update();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert!(child.contains::<_Component>());
	}

	#[test]
	fn remove_spawn_command() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(SpawnChildren(|parent| {
				parent.spawn(_Component);
			}))
			.id();

		app.update();

		assert!(!app.world().entity(entity).contains::<SpawnChildren>());
	}
}

#[cfg(test)]
mod test_spawn_children_from_parent {
	use super::*;
	use crate::{assert_count, get_children, test_tools::utils::SingleThreadedApp};

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Parent;

	#[derive(Component, Debug, PartialEq)]
	struct _Component(_Parent);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, SpawnChildrenFromParent::<_Parent>::system);

		app
	}

	#[test]
	fn spawn_children() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				SpawnChildrenFromParent(|entity, parent: &_Parent| {
					entity.spawn(_Component(parent.clone()));
				}),
				_Parent,
			))
			.id();

		app.update();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert!(child.contains::<_Component>());
	}

	#[test]
	fn remove_spawn_command() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				SpawnChildrenFromParent(|entity, parent: &_Parent| {
					entity.spawn(_Component(parent.clone()));
				}),
				_Parent,
			))
			.id();

		app.update();

		assert!(
			!app.world()
				.entity(entity)
				.contains::<SpawnChildrenFromParent<_Parent>>()
		);
	}
}
