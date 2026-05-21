use crate::components::collider::{ChildColliderOf, ColliderRoot};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl ColliderRoot {
	pub(crate) fn link_children(
		trigger: On<Add, Collider>,
		mut commands: ZyheedaCommands,
		collider_roots: Query<Entity, With<Self>>,
		ancestors: Query<&ChildOf>,
	) {
		let get_target_in_ancestor_of = |entity| {
			ancestors
				.iter_ancestors(entity)
				.find(|ancestor| collider_roots.contains(*ancestor))
		};
		let entity = trigger.entity;
		let Some(target) = get_target_in_ancestor_of(entity) else {
			return;
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(ChildColliderOf(target));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(ColliderRoot::link_children);

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

		assert_eq!(
			Some(&ChildColliderOf(entity)),
			app.world().entity(child).get::<ChildColliderOf>()
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

		assert_eq!(
			Some(&ChildColliderOf(entity)),
			app.world().entity(child_child).get::<ChildColliderOf>()
		);
	}

	#[test]
	fn act_only_once() {
		#[derive(Resource, Debug, PartialEq)]
		struct _Changed(bool);

		impl _Changed {
			fn system(mut commands: Commands, colliders: Query<(), Changed<ChildColliderOf>>) {
				commands.insert_resource(_Changed(colliders.iter().count() > 0));
			}
		}

		let mut app = setup();
		app.add_systems(Update, _Changed::system);

		let entity = app
			.world_mut()
			.spawn((ColliderRoot, Collider::default()))
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
