use crate::components::{
	blocker_types::BlockerTypes,
	center_offset::CenterOffset,
	collider::{
		ColliderShape,
		GENERIC_COLLISION_GROUP,
		MOUSE_HOVERABLE_GROUP,
		RAY_GROUP,
		TERRAIN_GROUP,
	},
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
#[require(CenterOffset)]
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
		let groups = match physics_type {
			PhysicsType::Agent => GENERIC_COLLISION_GROUP | MOUSE_HOVERABLE_GROUP,
			PhysicsType::Terrain => GENERIC_COLLISION_GROUP | TERRAIN_GROUP,
		};

		match self.0.physics_type {
			PhysicsType::Agent => {
				entity.try_insert((
					RigidBody::KinematicPositionBased,
					ActiveEvents::COLLISION_EVENTS,
					ActiveCollisionTypes::all(),
					Self::agent_controller(),
				));
			}
			PhysicsType::Terrain => {
				entity.try_insert(RigidBody::Fixed);
			}
		};

		entity.try_insert((
			BlockerTypes(self.0.blocker_types.clone()),
			CollisionGroups::new(groups, GENERIC_COLLISION_GROUP | RAY_GROUP),
			ColliderShape::from(self.0.shape),
		));

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::Units,
		traits::{
			handles_physics::physical_bodies::{Blocker, Body, Shape},
			prefab::AddPrefabObserver,
		},
	};
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
	fn insert_physics_constraints(
		physics_type: PhysicsType,
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
				Body::from_shape(shape).with_physics_type(physics_type),
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

	#[test]
	fn insert_blocker_types() {
		let mut app = setup();
		let blocks = [Blocker::Force, Blocker::Physical];
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(shape).with_blocker_types(blocks),
			))
			.id();

		assert_eq!(
			Some(&BlockerTypes::from(blocks)),
			app.world().entity(entity).get::<BlockerTypes>(),
		);
	}

	#[test_case(PhysicsType::Terrain, GENERIC_COLLISION_GROUP | TERRAIN_GROUP; "terrain")]
	#[test_case(PhysicsType::Agent, GENERIC_COLLISION_GROUP | MOUSE_HOVERABLE_GROUP; "agent")]
	fn insert_collision_groups(physics_type: PhysicsType, memberships: Group) {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(shape).with_physics_type(physics_type),
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
}
