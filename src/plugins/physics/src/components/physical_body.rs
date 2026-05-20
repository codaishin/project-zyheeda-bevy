use crate::components::{
	blocker_types::BlockerTypes,
	collider::{
		ColliderShape,
		GENERIC_COLLISION_GROUP,
		INTERACTIVE_GROUP,
		MOUSE_HOVERABLE_GROUP,
		RAY_GROUP,
		TERRAIN_GROUP,
	},
	offset::{AimOffset, CenterOffset},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use bevy_rapier3d::prelude::*;
use common::{
	errors::Unreachable,
	traits::{
		handles_physics::physical_bodies::{Body, PhysicsType},
		prefab::{Prefab, PrefabEntityCommands},
	},
};

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
#[require(AimOffset, CenterOffset)]
pub struct PhysicalBody(pub(crate) Body);

impl PhysicalBody {
	fn agent_controller() -> KinematicCharacterController {
		KinematicCharacterController { ..default() }
	}
}

impl From<Body> for PhysicalBody {
	fn from(body: Body) -> Self {
		Self(body)
	}
}

impl Prefab<()> for PhysicalBody {
	type TError = Unreachable;
	type TSystemParam<'w, 's> = ();

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: StaticSystemParam<()>,
	) -> Result<(), Self::TError> {
		let Self(Body { physics_type, .. }) = self;

		let specific_memberships = match physics_type {
			PhysicsType::Agent(blockers) => {
				entity.try_insert((
					RigidBody::KinematicPositionBased,
					BlockerTypes(blockers.clone()),
					ActiveEvents::COLLISION_EVENTS,
					ActiveCollisionTypes::all(),
					Self::agent_controller(),
				));
				MOUSE_HOVERABLE_GROUP
			}
			PhysicsType::Terrain(blockers) => {
				entity.try_insert((RigidBody::Fixed, BlockerTypes(blockers.clone())));
				TERRAIN_GROUP
			}
			PhysicsType::InteractiveFrame => {
				entity.try_insert((RigidBody::Fixed, Sensor));
				INTERACTIVE_GROUP
			}
		};

		entity.try_insert((
			CollisionGroups {
				memberships: GENERIC_COLLISION_GROUP | specific_memberships,
				filters: GENERIC_COLLISION_GROUP | RAY_GROUP,
			},
			ColliderShape::from(self.0.shape),
		));

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::collider::INTERACTIVE_GROUP;
	use common::{
		tools::Units,
		traits::{
			handles_physics::physical_bodies::{Blocker, Body, Shape},
			prefab::AddPrefabObserver,
		},
	};
	use std::collections::HashSet;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_prefab_observer::<PhysicalBody, ()>();

		app
	}

	#[test]
	fn insert_collider() {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(Body::from_shape(shape)))
			.id();

		assert_eq!(
			Some(&ColliderShape::from(shape)),
			app.world().entity(entity).get::<ColliderShape>(),
		);
	}

	#[test_case(PhysicsType::Terrain, RigidBody::Fixed, false, None, None; "terrain")]
	#[test_case(PhysicsType::Agent, RigidBody::KinematicPositionBased, true, Some(&ActiveEvents::COLLISION_EVENTS), Some(&ActiveCollisionTypes::all()); "agent")]
	#[test_case(|_| PhysicsType::InteractiveFrame, RigidBody::Fixed, false, None, None; "interactive frame")]
	fn insert_physics_constraints(
		physics_type: fn(HashSet<Blocker>) -> PhysicsType,
		rigid_body: RigidBody,
		has_character_controller: bool,
		active_events: Option<&ActiveEvents>,
		collision_types: Option<&ActiveCollisionTypes>,
	) {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(shape).with_physics_type(physics_type(HashSet::default())),
			))
			.id();

		assert_eq!(
			(
				Some(&rigid_body),
				has_character_controller,
				active_events,
				collision_types
			),
			(
				app.world().entity(entity).get::<RigidBody>(),
				app.world()
					.entity(entity)
					.contains::<KinematicCharacterController>(),
				app.world().entity(entity).get::<ActiveEvents>(),
				app.world().entity(entity).get::<ActiveCollisionTypes>(),
			)
		);
	}

	#[test_case(PhysicsType::Terrain, Some; "terrain")]
	#[test_case(PhysicsType::Agent, Some; "agent")]
	#[test_case(|_| PhysicsType::InteractiveFrame, |_| None; "interactive frame")]
	fn insert_blocker_types(
		physics_type: fn(HashSet<Blocker>) -> PhysicsType,
		blocks: fn(HashSet<Blocker>) -> Option<HashSet<Blocker>>,
	) {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(Body::from_shape(shape).with_physics_type(
				physics_type(HashSet::from([Blocker::Force, Blocker::Physical])),
			)))
			.id();

		assert_eq!(
			blocks(HashSet::from([Blocker::Force, Blocker::Physical])).map(BlockerTypes::from),
			app.world().entity(entity).get::<BlockerTypes>().cloned(),
		);
	}

	#[test_case(PhysicsType::Terrain, GENERIC_COLLISION_GROUP | TERRAIN_GROUP; "terrain")]
	#[test_case(PhysicsType::Agent, GENERIC_COLLISION_GROUP | MOUSE_HOVERABLE_GROUP; "agent")]
	#[test_case(|_| PhysicsType::InteractiveFrame, GENERIC_COLLISION_GROUP | INTERACTIVE_GROUP; "interactive frame")]
	fn insert_collision_groups(
		physics_type: fn(HashSet<Blocker>) -> PhysicsType,
		memberships: Group,
	) {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(shape).with_physics_type(physics_type(HashSet::default())),
			))
			.id();

		assert_eq!(
			Some(&CollisionGroups {
				memberships,
				filters: GENERIC_COLLISION_GROUP | RAY_GROUP
			}),
			app.world().entity(entity).get::<CollisionGroups>(),
		);
	}

	#[test_case(PhysicsType::Terrain, None; "terrain")]
	#[test_case(PhysicsType::Agent, None; "agent")]
	#[test_case(|_| PhysicsType::InteractiveFrame, Some(&Sensor); "interactive object")]
	fn insert_sensor(physics_type: fn(HashSet<Blocker>) -> PhysicsType, sensor: Option<&Sensor>) {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(shape).with_physics_type(physics_type(HashSet::default())),
			))
			.id();

		assert_eq!(sensor, app.world().entity(entity).get::<Sensor>());
	}
}
