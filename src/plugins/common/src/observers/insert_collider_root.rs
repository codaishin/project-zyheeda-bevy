use crate::{components::collider_root::ColliderRoot, traits::try_insert_on::TryInsertOn};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

impl ColliderRoot {
	pub(crate) fn insert(
		trigger: Trigger<OnAdd, Collider>,
		mut commands: Commands,
		rigid_bodies: Query<Entity, With<RigidBody>>,
		children: Query<&ChildOf>,
	) {
		let get_rigid_body_in_ancestor = |entity| {
			children
				.iter_ancestors(entity)
				.find(|ancestor| rigid_bodies.contains(*ancestor))
		};
		let get_rigid_body = |entity| {
			rigid_bodies
				.get(entity)
				.ok()
				.or_else(|| get_rigid_body_in_ancestor(entity))
		};
		let entity = trigger.target();
		let Some(rigid_body_entity) = get_rigid_body(entity) else {
			return;
		};

		commands.try_insert_on(entity, ColliderRoot(rigid_body_entity));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(ColliderRoot::insert);

		app
	}

	#[test]
	fn insert_when_collider_and_rigid_body_on_same_entity() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn((RigidBody::default(), Collider::default()))
			.id();

		assert_eq!(
			Some(&ColliderRoot(entity)),
			app.world().entity(entity).get::<ColliderRoot>()
		);
	}

	#[test]
	fn insert_when_collider_on_child_of_rigid_body() {
		let mut app = setup();
		let entity = app.world_mut().spawn(RigidBody::default()).id();

		let child = app
			.world_mut()
			.spawn((Collider::default(), ChildOf(entity)))
			.id();

		assert_eq!(
			Some(&ColliderRoot(entity)),
			app.world().entity(child).get::<ColliderRoot>()
		);
	}

	#[test]
	fn insert_when_collider_on_child_of_child_of_rigid_body() {
		let mut app = setup();
		let entity = app.world_mut().spawn(RigidBody::default()).id();
		let child = app.world_mut().spawn(ChildOf(entity)).id();

		let child_child = app
			.world_mut()
			.spawn((Collider::default(), ChildOf(child)))
			.id();

		assert_eq!(
			Some(&ColliderRoot(entity)),
			app.world().entity(child_child).get::<ColliderRoot>()
		);
	}

	#[test]
	fn act_only_once() {
		#[derive(Resource, Debug, PartialEq)]
		struct _Changed(bool);

		impl _Changed {
			fn system(mut commands: Commands, collider_roots: Query<(), Changed<ColliderRoot>>) {
				commands.insert_resource(_Changed(collider_roots.iter().count() > 0));
			}
		}

		let mut app = setup();
		app.add_systems(Update, _Changed::system);

		let entity = app
			.world_mut()
			.spawn((RigidBody::default(), Collider::default()))
			.id();
		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Collider::default());
		app.update();

		assert_eq!(
			Some(&_Changed(false)),
			app.world().get_resource::<_Changed>()
		);
	}
}
