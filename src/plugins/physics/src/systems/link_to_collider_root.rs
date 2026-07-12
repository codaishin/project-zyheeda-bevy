use crate::components::collider::{ColliderOf, ColliderRoot};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl ColliderRoot {
	pub(crate) fn link_children(
		mut commands: ZyheedaCommands,
		roots: Query<(), With<Self>>,
		colliders: Query<Entity, Added<Collider>>,
		ancestors: Query<&ChildOf>,
	) {
		for entity in colliders {
			let get_target_in_ancestor_of = |entity| {
				ancestors
					.iter_ancestors(entity)
					.find(|ancestor| roots.contains(*ancestor))
			};
			let Some(target) = get_target_in_ancestor_of(entity) else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(ColliderOf(target));
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				ColliderRoot::link_children,
				IsChanged::<ColliderOf>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn insert_when_collider_on_child_of_target() {
		let mut app = setup();
		let entity = app.world_mut().spawn(ColliderRoot).id();
		let child = app
			.world_mut()
			.spawn((Collider::default(), ChildOf(entity)))
			.id();

		app.update();

		assert_eq!(
			Some(&ColliderOf(entity)),
			app.world().entity(child).get::<ColliderOf>()
		);
	}

	#[test]
	fn insert_when_collider_on_child_of_child_of_target() {
		let mut app = setup();
		let entity = app.world_mut().spawn(ColliderRoot).id();
		let child = app.world_mut().spawn(ChildOf(entity)).id();
		let child_child = app
			.world_mut()
			.spawn((Collider::default(), ChildOf(child)))
			.id();

		app.update();

		assert_eq!(
			Some(&ColliderOf(entity)),
			app.world().entity(child_child).get::<ColliderOf>()
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();

		let entity = app.world_mut().spawn(ColliderRoot).id();
		let child = app
			.world_mut()
			.spawn((Collider::default(), ChildOf(entity)))
			.id();
		app.update();
		app.world_mut()
			.entity_mut(child)
			.insert(Collider::default());
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(child)
				.get::<IsChanged<ColliderOf>>()
		);
	}
}
