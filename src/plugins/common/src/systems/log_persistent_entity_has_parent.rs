use crate::{
	components::persistent_entity::PersistentEntity,
	error_logger::Log,
	errors::{ErrorData, Level},
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use std::fmt::Display;

impl PersistentEntity {
	pub(crate) fn has_parent<TLogger>(
		logger: StaticSystemParam<TLogger>,
		entities: Query<(&PersistentEntity, &ChildOf), Changed<ChildOf>>,
	) where
		TLogger: for<'w, 's> SystemParam<Item<'w, 's>: Log>,
	{
		for (entity, ChildOf(parent)) in entities {
			logger.log(PersistentEntityHasParent {
				entity: *entity,
				parent: *parent,
			});
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct PersistentEntityHasParent {
	entity: PersistentEntity,
	parent: Entity,
}

impl Display for PersistentEntityHasParent {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{:?}: has a parent {:?}. This will result in undefined behavior when saving/loading",
			self.entity, self.parent,
		)
	}
}

impl ErrorData for PersistentEntityHasParent {
	fn level(&self) -> Level {
		Level::Warning
	}

	fn label() -> impl Display {
		"Persistent Entity Has Parent"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use std::{any::Any, sync::RwLock};
	use testing::SingleThreadedApp;

	#[derive(Resource)]
	struct _Logger(RwLock<Vec<PersistentEntityHasParent>>);

	impl Log for _Logger {
		fn log<TError>(&self, error: TError)
		where
			TError: ErrorData,
		{
			let Ok(mut lock) = self.0.write() else {
				return;
			};

			let error = (&error as &dyn Any)
				.downcast_ref::<PersistentEntityHasParent>()
				.unwrap();

			lock.push(*error);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(_Logger(RwLock::new(vec![])));
		app.add_systems(Update, PersistentEntity::has_parent::<Res<_Logger>>);

		app
	}

	#[test]
	fn log_parent() {
		let mut app = setup();
		let entity = PersistentEntity::default();
		let parent = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((ChildOf(parent), entity));

		app.update();

		assert_eq!(
			vec![PersistentEntityHasParent { entity, parent }],
			*app.world().resource::<_Logger>().0.read().unwrap(),
		);
	}

	#[test]
	fn log_only_once() {
		let mut app = setup();
		let entity = PersistentEntity::default();
		let parent = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((ChildOf(parent), entity));

		app.update();
		app.update();

		assert_eq!(
			vec![PersistentEntityHasParent { entity, parent }],
			*app.world().resource::<_Logger>().0.read().unwrap(),
		);
	}

	#[test]
	fn log_again_if_parent_changed() {
		let mut app = setup();
		let entity = PersistentEntity::default();
		let parent_a = app.world_mut().spawn_empty().id();
		let parent_b = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn((ChildOf(parent_a), entity)).id();

		app.update();
		app.world_mut().entity_mut(child).insert(ChildOf(parent_b));
		app.update();

		assert_eq!(
			vec![
				PersistentEntityHasParent {
					entity,
					parent: parent_a
				},
				PersistentEntityHasParent {
					entity,
					parent: parent_b
				},
			],
			*app.world().resource::<_Logger>().0.read().unwrap(),
		);
	}
}
