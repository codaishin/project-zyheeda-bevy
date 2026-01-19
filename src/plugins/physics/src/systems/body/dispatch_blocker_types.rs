use crate::components::{
	blocker_types::BlockerTypes,
	collider::Colliders,
	physical_body::PhysicalBody,
};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, handles_physics::physical_bodies::Body},
	zyheeda_commands::ZyheedaCommands,
};

impl PhysicalBody {
	pub(crate) fn dispatch_blocker_types(
		mut commands: ZyheedaCommands,
		colliders: Query<(&Colliders, &Self), Changed<Colliders>>,
	) {
		for (colliders, Self(Body { blocker_types, .. })) in &colliders {
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
	use crate::components::collider::ColliderOf;
	use common::{
		tools::Units,
		traits::handles_physics::physical_bodies::{Blocker, Body, Shape},
	};
	use std::{collections::HashSet, sync::LazyLock};
	use testing::SingleThreadedApp;

	static SHAPE: LazyLock<Shape> = LazyLock::new(|| Shape::Sphere {
		radius: Units::from(42.),
	});

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, PhysicalBody::dispatch_blocker_types);

		app
	}

	#[test]
	fn dispatch_blocker_types() {
		let mut app = setup();
		let blockers = HashSet::from([Blocker::Force, Blocker::Physical]);
		let root = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(*SHAPE).with_blocker_types(blockers.clone()),
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
			.spawn(PhysicalBody(
				Body::from_shape(*SHAPE).with_blocker_types(blockers.clone()),
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
			.spawn(PhysicalBody(
				Body::from_shape(*SHAPE).with_blocker_types(blockers.clone()),
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
}
