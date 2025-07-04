use crate::{
	components::collider_relationship::{ColliderOfInteractionTarget, InteractionTarget},
	traits::try_insert_on::TryInsertOn,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

impl ColliderOfInteractionTarget {
	pub(crate) fn link(
		trigger: Trigger<OnAdd, Collider>,
		mut commands: Commands,
		collider_roots: Query<Entity, With<InteractionTarget>>,
		ancestors: Query<&ChildOf>,
	) {
		let get_target_in_ancestor_of = |entity| {
			ancestors
				.iter_ancestors(entity)
				.find(|ancestor| collider_roots.contains(*ancestor))
		};
		let entity = trigger.target();
		let Some(target) = get_target_in_ancestor_of(entity) else {
			return;
		};

		commands.try_insert_on(entity, ColliderOfInteractionTarget(target));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::collider_relationship::InteractionTarget;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(ColliderOfInteractionTarget::link);

		app
	}

	#[test]
	fn insert_when_collider_on_child_of_target() {
		let mut app = setup();
		let entity = app.world_mut().spawn(InteractionTarget).id();

		let child = app
			.world_mut()
			.spawn((Collider::default(), ChildOf(entity)))
			.id();

		assert_eq!(
			Some(&ColliderOfInteractionTarget(entity)),
			app.world()
				.entity(child)
				.get::<ColliderOfInteractionTarget>()
		);
	}

	#[test]
	fn insert_when_collider_on_child_of_child_of_target() {
		let mut app = setup();
		let entity = app.world_mut().spawn(InteractionTarget).id();
		let child = app.world_mut().spawn(ChildOf(entity)).id();

		let child_child = app
			.world_mut()
			.spawn((Collider::default(), ChildOf(child)))
			.id();

		assert_eq!(
			Some(&ColliderOfInteractionTarget(entity)),
			app.world()
				.entity(child_child)
				.get::<ColliderOfInteractionTarget>()
		);
	}

	#[test]
	fn act_only_once() {
		#[derive(Resource, Debug, PartialEq)]
		struct _Changed(bool);

		impl _Changed {
			fn system(
				mut commands: Commands,
				colliders: Query<(), Changed<ColliderOfInteractionTarget>>,
			) {
				commands.insert_resource(_Changed(colliders.iter().count() > 0));
			}
		}

		let mut app = setup();
		app.add_systems(Update, _Changed::system);

		let entity = app
			.world_mut()
			.spawn((InteractionTarget, Collider::default()))
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
