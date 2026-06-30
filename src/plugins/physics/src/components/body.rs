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
	collision_domains::{Interactive, Physical},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use bevy_rapier3d::prelude::*;
use common::{
	errors::Unreachable,
	traits::{
		handles_physics::physical_bodies::{BodyConfig, PhysicsType},
		prefab::{Prefab, PrefabEntityCommands},
	},
};

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub struct Body(pub(crate) BodyConfig);

impl Body {
	fn agent() -> impl Bundle {
		(
			Physical::Contact,
			RigidBody::KinematicPositionBased,
			CollidingEntities::default(),
			ActiveEvents::COLLISION_EVENTS,
			ActiveCollisionTypes::all(),
			KinematicCharacterController::default(),
			CollisionGroups {
				memberships: generic_and(MOUSE_HOVERABLE_GROUP),
				filters: generic_and(RAY_GROUP),
			},
		)
	}

	fn terrain() -> impl Bundle {
		(
			Physical::Contact,
			RigidBody::Fixed,
			CollisionGroups {
				memberships: generic_and(TERRAIN_GROUP),
				filters: generic_and(RAY_GROUP),
			},
		)
	}

	fn interactive() -> impl Bundle {
		(
			Interactive,
			Sensor,
			CollidingEntities::default(),
			ActiveEvents::COLLISION_EVENTS,
			ActiveCollisionTypes::all(),
			CollisionGroups {
				memberships: INTERACTIVE_GROUP,
				filters: INTERACTIVE_GROUP | RAY_GROUP,
			},
		)
	}
}

impl From<BodyConfig> for Body {
	fn from(body: BodyConfig) -> Self {
		Self(body)
	}
}

impl Prefab<()> for Body {
	type TError = Unreachable;
	type TSystemParam = ();

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: StaticSystemParam<()>,
	) -> Result<(), Self::TError> {
		let Self(BodyConfig {
			physics_type,
			shape,
			sub_frames,
		}) = self;

		match physics_type {
			PhysicsType::Agent(blockers) => {
				entity.try_insert((Self::agent(), BlockerTypes(blockers.clone())));
			}
			PhysicsType::Terrain(blockers) => {
				entity.try_insert((Self::terrain(), BlockerTypes(blockers.clone())));
			}
			PhysicsType::InteractiveFrame => {
				entity.try_insert((Self::interactive(), RigidBody::Fixed));
			}
		};

		entity.try_insert(ColliderShape::from(*shape));

		for sub_frame in sub_frames {
			entity.with_child((
				Self::interactive(),
				Transform::from_xyz(0., 0., -*sub_frame.forward_offset),
				ColliderShape::from(sub_frame.shape),
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
	use common::{
		tools::Units,
		traits::{
			handles_physics::physical_bodies::{Blocker, BodyConfig, Shape, ShapeParameters},
			prefab::AddPrefabObserver,
		},
	};
	use std::collections::HashSet;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_prefab_observer::<Body, ()>();

		app
	}

	mod body {
		use crate::components::collision_domains::Physical;

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
				.spawn(Body(BodyConfig::from_shape(shape)))
				.id();

			assert_eq!(
				Some(&ColliderShape::from(shape)),
				app.world().entity(entity).get::<ColliderShape>(),
			);
		}

		#[test_case(PhysicsType::Terrain, |e| e.contains::<Physical>(); "terrain as physical")]
		#[test_case(PhysicsType::Agent, |e| e.contains::<Physical>(); "agent as physical")]
		#[test_case(|_| PhysicsType::InteractiveFrame, |e| e.contains::<Interactive>(); "interactive frame as interactive")]
		fn mark(
			physics_type: fn(HashSet<Blocker>) -> PhysicsType,
			marker: fn(EntityWorldMut) -> bool,
		) {
			let mut app = setup();
			let shape = Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(42.),
			});

			let entity = app.world_mut().spawn(Body(
				BodyConfig::from_shape(shape).with_physics_type(physics_type(HashSet::default())),
			));

			assert!(marker(entity));
		}

		#[test_case(PhysicsType::Terrain, RigidBody::Fixed, false, false, None, None; "terrain")]
		#[test_case(PhysicsType::Agent, RigidBody::KinematicPositionBased, true, true, Some(&ActiveEvents::COLLISION_EVENTS), Some(&ActiveCollisionTypes::all()); "agent")]
		#[test_case(|_| PhysicsType::InteractiveFrame, RigidBody::Fixed, false, true, Some(&ActiveEvents::COLLISION_EVENTS), Some(&ActiveCollisionTypes::all()); "interactive frame")]
		fn insert_physics_constraints(
			physics_type: fn(HashSet<Blocker>) -> PhysicsType,
			rigid_body: RigidBody,
			has_character_controller: bool,
			has_colliding_entities: bool,
			active_events: Option<&ActiveEvents>,
			collision_types: Option<&ActiveCollisionTypes>,
		) {
			let mut app = setup();
			let shape = Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(42.),
			});

			let entity = app.world_mut().spawn(Body(
				BodyConfig::from_shape(shape).with_physics_type(physics_type(HashSet::default())),
			));

			assert_eq!(
				(
					Some(&rigid_body),
					has_character_controller,
					has_colliding_entities,
					active_events,
					collision_types
				),
				(
					entity.get::<RigidBody>(),
					entity.contains::<KinematicCharacterController>(),
					entity.contains::<CollidingEntities>(),
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
					.spawn(Body(BodyConfig::from_shape(shape).with_physics_type(
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

			let entity = app.world_mut().spawn(Body(
				BodyConfig::from_shape(shape).with_physics_type(physics_type(HashSet::default())),
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

			let entity = app.world_mut().spawn(Body(
				BodyConfig::from_shape(shape).with_physics_type(physics_type(HashSet::default())),
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
				.spawn(Body(
					BodyConfig::from_shape(Shape::StaticGltfMesh3d)
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
				.spawn(Body(
					BodyConfig::from_shape(Shape::StaticGltfMesh3d).with_sub_frames([
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
				.spawn(Body(
					BodyConfig::from_shape(Shape::StaticGltfMesh3d)
						.with_sub_frames([InteractiveFrame::from(shape)]),
				))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(
				(
					None,
					false,
					true,
					Some(&ActiveEvents::COLLISION_EVENTS),
					Some(&ActiveCollisionTypes::all()),
				),
				(
					child.get::<RigidBody>(),
					child.contains::<KinematicCharacterController>(),
					child.contains::<CollidingEntities>(),
					child.get::<ActiveEvents>(),
					child.get::<ActiveCollisionTypes>(),
				)
			);
		}

		#[test]
		fn mark_as_interactive() {
			let mut app = setup();
			let shape = ShapeParameters::Sphere {
				radius: Units::from(42.),
			};

			let entity = app
				.world_mut()
				.spawn(Body(
					BodyConfig::from_shape(Shape::StaticGltfMesh3d)
						.with_sub_frames([InteractiveFrame::from(shape)]),
				))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(Some(&Interactive), child.get::<Interactive>());
		}

		#[test]
		fn insert_collision_groups() {
			let mut app = setup();
			let shape = ShapeParameters::Sphere {
				radius: Units::from(42.),
			};
			let entity = app
				.world_mut()
				.spawn(Body(
					BodyConfig::from_shape(Shape::StaticGltfMesh3d)
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
				.spawn(Body(
					BodyConfig::from_shape(Shape::StaticGltfMesh3d)
						.with_sub_frames([InteractiveFrame::from(shape)]),
				))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(Some(&Sensor), child.get::<Sensor>());
		}
	}
}
