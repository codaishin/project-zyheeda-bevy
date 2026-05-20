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
		handles_physics::physical_bodies::{Body, InteractiveFrame, PhysicsType},
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
		let Self(Body {
			physics_type,
			shape,
			sub_frames,
		}) = self;

		let specific_memberships_group = match physics_type {
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
				entity.try_insert((
					RigidBody::Fixed,
					Sensor,
					ActiveEvents::COLLISION_EVENTS,
					ActiveCollisionTypes::all(),
				));
				INTERACTIVE_GROUP
			}
		};

		entity.try_insert((
			CollisionGroups {
				memberships: generic_and(specific_memberships_group),
				filters: generic_and(RAY_GROUP),
			},
			ColliderShape::from(*shape),
		));

		for InteractiveFrame(shape) in sub_frames {
			entity.with_child((
				ColliderShape::from(*shape),
				Sensor,
				ActiveEvents::COLLISION_EVENTS,
				ActiveCollisionTypes::all(),
				CollisionGroups {
					memberships: generic_and(INTERACTIVE_GROUP),
					filters: generic_and(RAY_GROUP),
				},
			));
		}

		Ok(())
	}
}

fn generic_and(group: Group) -> Group {
	GENERIC_COLLISION_GROUP | group
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::collider::INTERACTIVE_GROUP;
	use common::{
		tools::Units,
		traits::{
			handles_physics::physical_bodies::{Blocker, Body, Shape, ShapeParameters},
			prefab::AddPrefabObserver,
		},
	};
	use std::collections::HashSet;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_prefab_observer::<PhysicalBody, ()>();

		app
	}

	mod body {
		use super::*;
		use test_case::test_case;

		#[test]
		fn insert_collider() {
			let mut app = setup();
			let shape = Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(42.),
			});
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
		#[test_case(|_| PhysicsType::InteractiveFrame, RigidBody::Fixed, false, Some(&ActiveEvents::COLLISION_EVENTS), Some(&ActiveCollisionTypes::all()); "interactive frame")]
		fn insert_physics_constraints(
			physics_type: fn(HashSet<Blocker>) -> PhysicsType,
			rigid_body: RigidBody,
			has_character_controller: bool,
			active_events: Option<&ActiveEvents>,
			collision_types: Option<&ActiveCollisionTypes>,
		) {
			let mut app = setup();
			let shape = Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(42.),
			});
			let entity = app.world_mut().spawn(PhysicalBody(
				Body::from_shape(shape).with_physics_type(physics_type(HashSet::default())),
			));

			assert_eq!(
				(
					Some(&rigid_body),
					has_character_controller,
					active_events,
					collision_types
				),
				(
					entity.get::<RigidBody>(),
					entity.contains::<KinematicCharacterController>(),
					entity.get::<ActiveEvents>(),
					entity.get::<ActiveCollisionTypes>(),
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
			let shape = Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(42.),
			});
			let entity =
				app.world_mut()
					.spawn(PhysicalBody(Body::from_shape(shape).with_physics_type(
						physics_type(HashSet::from([Blocker::Force, Blocker::Physical])),
					)));

			assert_eq!(
				blocks(HashSet::from([Blocker::Force, Blocker::Physical])).map(BlockerTypes::from),
				entity.get::<BlockerTypes>().cloned(),
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
			let shape = Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(42.),
			});
			let entity = app.world_mut().spawn(PhysicalBody(
				Body::from_shape(shape).with_physics_type(physics_type(HashSet::default())),
			));

			assert_eq!(
				Some(&CollisionGroups {
					memberships,
					filters: GENERIC_COLLISION_GROUP | RAY_GROUP
				}),
				entity.get::<CollisionGroups>(),
			);
		}

		#[test_case(PhysicsType::Terrain, None; "terrain")]
		#[test_case(PhysicsType::Agent, None; "agent")]
		#[test_case(|_| PhysicsType::InteractiveFrame, Some(&Sensor); "interactive object")]
		fn insert_sensor(
			physics_type: fn(HashSet<Blocker>) -> PhysicsType,
			sensor: Option<&Sensor>,
		) {
			let mut app = setup();
			let shape = Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(42.),
			});

			let entity = app.world_mut().spawn(PhysicalBody(
				Body::from_shape(shape).with_physics_type(physics_type(HashSet::default())),
			));

			assert_eq!(sensor, entity.get::<Sensor>());
		}
	}

	mod sub_frames {
		use super::*;
		use common::traits::handles_physics::physical_bodies::InteractiveFrame;
		use testing::assert_children_count;

		#[test]
		fn insert_collider() {
			let mut app = setup();
			let shape = ShapeParameters::Sphere {
				radius: Units::from(42.),
			};
			let entity = app
				.world_mut()
				.spawn(PhysicalBody(
					Body::from_shape(Shape::StaticGltfMesh3d)
						.with_sub_frames([InteractiveFrame(shape)]),
				))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(
				Some(&ColliderShape::from(shape)),
				child.get::<ColliderShape>(),
			);
		}

		#[test]
		fn insert_physics_constraints() {
			let mut app = setup();
			let shape = ShapeParameters::Sphere {
				radius: Units::from(42.),
			};
			let entity = app
				.world_mut()
				.spawn(PhysicalBody(
					Body::from_shape(Shape::StaticGltfMesh3d)
						.with_sub_frames([InteractiveFrame(shape)]),
				))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(
				(
					None,
					false,
					Some(&ActiveEvents::COLLISION_EVENTS),
					Some(&ActiveCollisionTypes::all()),
				),
				(
					child.get::<RigidBody>(),
					child.contains::<KinematicCharacterController>(),
					child.get::<ActiveEvents>(),
					child.get::<ActiveCollisionTypes>(),
				)
			);
		}

		#[test]
		fn insert_collision_groups() {
			let mut app = setup();
			let shape = ShapeParameters::Sphere {
				radius: Units::from(42.),
			};
			let entity = app
				.world_mut()
				.spawn(PhysicalBody(
					Body::from_shape(Shape::StaticGltfMesh3d)
						.with_sub_frames([InteractiveFrame(shape)]),
				))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(
				Some(&CollisionGroups {
					memberships: GENERIC_COLLISION_GROUP | INTERACTIVE_GROUP,
					filters: GENERIC_COLLISION_GROUP | RAY_GROUP
				}),
				child.get::<CollisionGroups>(),
			);
		}

		#[test]
		fn insert_sensor() {
			let mut app = setup();
			let shape = ShapeParameters::Sphere {
				radius: Units::from(42.),
			};

			let entity = app
				.world_mut()
				.spawn(PhysicalBody(
					Body::from_shape(Shape::StaticGltfMesh3d)
						.with_sub_frames([InteractiveFrame(shape)]),
				))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(Some(&Sensor), child.get::<Sensor>());
		}
	}
}
