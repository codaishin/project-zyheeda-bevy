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
		let Self(Body {
			physics_type,
			shape,
			sub_frames,
		}) = self;

		match physics_type {
			PhysicsType::Agent(blockers) => {
				entity.try_insert((
					RigidBody::KinematicPositionBased,
					BlockerTypes(blockers.clone()),
					ActiveEvents::COLLISION_EVENTS,
					ActiveCollisionTypes::all(),
					Self::agent_controller(),
					CollisionGroups {
						memberships: generic_and(MOUSE_HOVERABLE_GROUP),
						filters: generic_and(RAY_GROUP),
					},
				));
			}
			PhysicsType::Terrain(blockers) => {
				entity.try_insert((
					RigidBody::Fixed,
					BlockerTypes(blockers.clone()),
					CollisionGroups {
						memberships: generic_and(TERRAIN_GROUP),
						filters: generic_and(RAY_GROUP),
					},
				));
			}
			PhysicsType::InteractiveFrame => {
				entity.try_insert((
					RigidBody::Fixed,
					Sensor,
					ActiveEvents::COLLISION_EVENTS,
					ActiveCollisionTypes::all(),
					CollisionGroups {
						memberships: INTERACTIVE_GROUP,
						filters: INTERACTIVE_GROUP | RAY_GROUP,
					},
				));
			}
		};

		entity.try_insert(ColliderShape::from(*shape));

		for sub_frame in sub_frames {
			entity.with_child((
				Transform::from_xyz(0., 0., -*sub_frame.forward_offset),
				ColliderShape::from(sub_frame.shape),
				Sensor,
				ActiveEvents::COLLISION_EVENTS,
				ActiveCollisionTypes::all(),
				CollisionGroups {
					memberships: INTERACTIVE_GROUP,
					filters: INTERACTIVE_GROUP | RAY_GROUP,
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

		#[test_case(PhysicsType::Terrain, generic_and(TERRAIN_GROUP) , generic_and(RAY_GROUP); "terrain")]
		#[test_case(PhysicsType::Agent, generic_and(MOUSE_HOVERABLE_GROUP), generic_and(RAY_GROUP); "agent")]
		#[test_case(|_| PhysicsType::InteractiveFrame, INTERACTIVE_GROUP, INTERACTIVE_GROUP | RAY_GROUP; "interactive frame")]
		fn insert_collision_groups(
			physics_type: fn(HashSet<Blocker>) -> PhysicsType,
			memberships: Group,
			filters: Group,
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
					filters
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
						.with_sub_frames([InteractiveFrame::from(shape)]),
				))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(
				Some(&ColliderShape::from(shape)),
				child.get::<ColliderShape>(),
			);
		}

		#[test]
		fn insert_transform() {
			let mut app = setup();
			let shape = ShapeParameters::Sphere {
				radius: Units::from(42.),
			};
			let entity = app
				.world_mut()
				.spawn(PhysicalBody(
					Body::from_shape(Shape::StaticGltfMesh3d).with_sub_frames([
						InteractiveFrame::from(shape).with_forward_offset(Units::from(11.)),
					]),
				))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(
				Some(&Transform::from_xyz(0., 0., -11.)),
				child.get::<Transform>(),
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
						.with_sub_frames([InteractiveFrame::from(shape)]),
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
						.with_sub_frames([InteractiveFrame::from(shape)]),
				))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(
				Some(&CollisionGroups {
					memberships: INTERACTIVE_GROUP,
					filters: INTERACTIVE_GROUP | RAY_GROUP,
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
						.with_sub_frames([InteractiveFrame::from(shape)]),
				))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(Some(&Sensor), child.get::<Sensor>());
		}
	}
}
