use crate::components::{
	blocker_types::BlockerTypes,
	colliders::{ColliderDefinition, Colliders},
};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, handles_physics::colliders::Collider},
	zyheeda_commands::ZyheedaCommands,
};

type CollidersOrDefinitionChanged = Or<(Changed<Colliders>, Changed<ColliderDefinition>)>;

impl Colliders {
	pub(crate) fn dispatch_blocker_types(
		mut commands: ZyheedaCommands,
		colliders: Query<(&Self, &ColliderDefinition), CollidersOrDefinitionChanged>,
	) {
		for (colliders, ColliderDefinition(Collider { blocker_types, .. })) in &colliders {
			for entity in colliders.iter() {
				commands.try_apply_on(&entity, |mut e| {
					e.try_insert(BlockerTypes(blocker_types.clone()));
				});
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::colliders::{ColliderDefinition, ColliderOf};
	use common::traits::handles_physics::colliders::{Blocker, Collider, Shape};
	use std::collections::HashSet;
	use testing::SingleThreadedApp;

	const SHAPE: Shape = Shape::Sphere { radius: 42. };

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Colliders::dispatch_blocker_types);

		app
	}

	#[test]
	fn dispatch_blocker_types() {
		let mut app = setup();
		let blockers = HashSet::from([Blocker::Force, Blocker::Physical]);
		let root = app
			.world_mut()
			.spawn(ColliderDefinition(
				Collider::from_shape(SHAPE).with_blocker_types(blockers.clone()),
			))
			.id();
		let collider = app.world_mut().spawn(ColliderOf(root)).id();

		app.update();

		assert_eq!(
			Some(&BlockerTypes(blockers)),
			app.world().entity(collider).get::<BlockerTypes>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let blockers = HashSet::from([Blocker::Force, Blocker::Physical]);
		let root = app
			.world_mut()
			.spawn(ColliderDefinition(
				Collider::from_shape(SHAPE).with_blocker_types(blockers.clone()),
			))
			.id();
		let collider = app.world_mut().spawn(ColliderOf(root)).id();

		app.update();
		app.world_mut()
			.entity_mut(collider)
			.remove::<BlockerTypes>();
		app.update();

		assert_eq!(None, app.world().entity(collider).get::<BlockerTypes>());
	}

	#[test]
	fn act_again_if_colliders_changed() {
		let mut app = setup();
		let blockers = HashSet::from([Blocker::Force, Blocker::Physical]);
		let root = app
			.world_mut()
			.spawn(ColliderDefinition(
				Collider::from_shape(SHAPE).with_blocker_types(blockers.clone()),
			))
			.id();
		let collider = app.world_mut().spawn(ColliderOf(root)).id();

		app.update();
		app.world_mut()
			.entity_mut(collider)
			.remove::<BlockerTypes>();
		app.world_mut()
			.entity_mut(root)
			.get_mut::<Colliders>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&BlockerTypes(blockers)),
			app.world().entity(collider).get::<BlockerTypes>()
		);
	}

	#[test]
	fn act_again_if_collider_definition_changed() {
		let mut app = setup();
		let blockers = HashSet::from([Blocker::Force, Blocker::Physical]);
		let root = app
			.world_mut()
			.spawn(ColliderDefinition(
				Collider::from_shape(SHAPE).with_blocker_types(blockers.clone()),
			))
			.id();
		let collider = app.world_mut().spawn(ColliderOf(root)).id();

		app.update();
		app.world_mut()
			.entity_mut(collider)
			.remove::<BlockerTypes>();
		app.world_mut()
			.entity_mut(root)
			.get_mut::<ColliderDefinition>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&BlockerTypes(blockers)),
			app.world().entity(collider).get::<BlockerTypes>()
		);
	}
}
