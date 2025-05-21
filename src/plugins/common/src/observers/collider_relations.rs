use crate::{components::collider_relations::ChildColliderOf, traits::try_insert_on::TryInsertOn};
use bevy::prelude::*;

impl ChildColliderOf {
	pub(crate) fn update_child_relation(
		trigger: Trigger<OnInsert, Self>,
		mut commands: Commands,
		child_colliders: Query<(Entity, &Self)>,
	) {
		let Ok((entity, ChildColliderOf(root))) = child_colliders.get(trigger.target()) else {
			return;
		};

		commands.try_insert_on(entity, ChildOf(*root));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		assert_count,
		components::collider_relations::ChildColliders,
		get_children,
		test_tools::utils::SingleThreadedApp,
	};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(ChildColliderOf::update_child_relation);

		app
	}

	#[test]
	fn insert_child_of() {
		let mut app = setup();

		let entity = app.world_mut().spawn(related!(ChildColliders[()])).id();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(
			Some(&ChildColliderOf(entity)),
			child.get::<ChildColliderOf>()
		);
	}

	#[test]
	fn update_child_of() {
		let mut app = setup();

		let entity_a = app.world_mut().spawn_empty().id();
		let entity_b = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn(ChildColliderOf(entity_a)).id();
		app.world_mut()
			.entity_mut(child)
			.insert(ChildColliderOf(entity_b));

		assert_count!(0, get_children!(app, entity_a));
		let [child] = assert_count!(1, get_children!(app, entity_b));
		assert_eq!(
			Some(&ChildColliderOf(entity_b)),
			child.get::<ChildColliderOf>()
		);
	}
}
