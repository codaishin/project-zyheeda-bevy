use crate::{
	components::child_of_persistent::ChildOfPersistent,
	resources::persistent_entities::PersistentEntities,
	traits::try_insert_on::TryInsertOn,
};
use bevy::prelude::*;

impl ChildOfPersistent {
	pub(crate) fn insert_child_of(
		trigger: Trigger<OnInsert, Self>,
		mut commands: Commands,
		mut persistent_entities: ResMut<PersistentEntities>,
		children_of_persistent: Query<&Self>,
	) {
		let child = trigger.target();
		let Ok(ChildOfPersistent(persistent_parent)) = children_of_persistent.get(child) else {
			return;
		};
		let Some(parent) = persistent_entities.get_entity(persistent_parent) else {
			return;
		};

		commands.try_insert_on(child, ChildOf(parent));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::persistent_entity::PersistentEntity,
		traits::register_persistent_entities::RegisterPersistentEntities,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.add_observer(ChildOfPersistent::insert_child_of);

		app
	}

	#[test]
	fn insert_child_of() {
		let mut app = setup();
		let persistent_parent = PersistentEntity::default();
		let parent = app.world_mut().spawn(persistent_parent).id();

		let child = app.world_mut().spawn(ChildOfPersistent(persistent_parent));

		assert_eq!(Some(&ChildOf(parent)), child.get::<ChildOf>());
	}

	#[test]
	fn insert_child_of_when_parent_changed() {
		let mut app = setup();
		let persistent_parent_a = PersistentEntity::default();
		app.world_mut().spawn(persistent_parent_a);
		let persistent_parent_b = PersistentEntity::default();
		let parent_b = app.world_mut().spawn(persistent_parent_b).id();

		let mut child = app
			.world_mut()
			.spawn(ChildOfPersistent(persistent_parent_a));
		child.insert(ChildOfPersistent(persistent_parent_b));

		assert_eq!(Some(&ChildOf(parent_b)), child.get::<ChildOf>());
	}
}
