use crate::components::{
	blocker_types::BlockerTypes,
	collider::{
		ChildColliderOf,
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

	fn interactive_of(entity: Entity) -> impl Bundle {
		(
			Interactive,
			Sensor,
			ChildColliderOf(entity),
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
		let Self(BodyConfig { core, sub_frames }) = self;

		if let Some(core) = core {
			match core.physics_type {
				PhysicsType::Agent(ref blockers) => {
					entity.try_insert((Self::agent(), BlockerTypes(blockers.clone())));
				}
				PhysicsType::Terrain(ref blockers) => {
					entity.try_insert((Self::terrain(), BlockerTypes(blockers.clone())));
				}
			};

			entity.try_insert(ColliderShape::from(core.shape));
		}

		for sub_frame in sub_frames {
			entity.with_child((
				Self::interactive_of(entity.entity_id()),
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
		use super::*;
		use crate::components::collision_domains::Physical;
		use common::traits::handles_physics::physical_bodies::Core;
		use test_case::test_case;

		#[test]
		fn insert_collider() {
			let mut app = setup();
			let shape = Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(42.),
			});

			let entity = app
				.world_mut()
				.spawn(Body(BodyConfig {
					core: Some(Core { shape, ..default() }),
					..default()
				}))
				.id();

			assert_eq!(
				Some(&ColliderShape::from(shape)),
				app.world().entity(entity).get::<ColliderShape>(),
			);
		}

		#[test_case(PhysicsType::Terrain, |e| e.contains::<Physical>(); "terrain as physical")]
		#[test_case(PhysicsType::Agent, |e| e.contains::<Physical>(); "agent as physical")]
		fn mark(
			physics_type: fn(HashSet<Blocker>) -> PhysicsType,
			marker: fn(EntityWorldMut) -> bool,
		) {
			let mut app = setup();
			let shape = Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(42.),
			});

			let entity = app.world_mut().spawn(Body(BodyConfig {
				core: Some(Core {
					shape,
					physics_type: physics_type(HashSet::default()),
				}),
				..default()
			}));

			assert!(marker(entity));
		}

		#[test_case(PhysicsType::Terrain, RigidBody::Fixed, false, false, None, None; "terrain")]
		#[test_case(PhysicsType::Agent, RigidBody::KinematicPositionBased, true, true, Some(&ActiveEvents::COLLISION_EVENTS), Some(&ActiveCollisionTypes::all()); "agent")]
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

			let entity = app.world_mut().spawn(Body(BodyConfig {
				core: Some(Core {
					shape,
					physics_type: physics_type(HashSet::new()),
				}),
				..default()
			}));

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
		fn insert_blocker_types(
			physics_type: fn(HashSet<Blocker>) -> PhysicsType,
			blocks: fn(HashSet<Blocker>) -> Option<HashSet<Blocker>>,
		) {
			let mut app = setup();
			let shape = Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(42.),
			});

			let entity = app.world_mut().spawn(Body(BodyConfig {
				core: Some(Core {
					shape,
					physics_type: physics_type(HashSet::from([Blocker::Force, Blocker::Physical])),
				}),
				..default()
			}));

			assert_eq!(
				blocks(HashSet::from([Blocker::Force, Blocker::Physical])).map(BlockerTypes::from),
				entity.get::<BlockerTypes>().cloned(),
			);
		}

		#[test_case(PhysicsType::Terrain, generic_and(TERRAIN_GROUP) , generic_and(RAY_GROUP); "terrain")]
		#[test_case(PhysicsType::Agent, generic_and(MOUSE_HOVERABLE_GROUP), generic_and(RAY_GROUP); "agent")]
		fn insert_collision_groups(
			physics_type: fn(HashSet<Blocker>) -> PhysicsType,
			memberships: Group,
			filters: Group,
		) {
			let mut app = setup();
			let shape = Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(42.),
			});

			let entity = app.world_mut().spawn(Body(BodyConfig {
				core: Some(Core {
					shape,
					physics_type: physics_type(HashSet::new()),
				}),
				..default()
			}));

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
		fn insert_sensor(
			physics_type: fn(HashSet<Blocker>) -> PhysicsType,
			sensor: Option<&Sensor>,
		) {
			let mut app = setup();
			let shape = Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(42.),
			});

			let entity = app.world_mut().spawn(Body(BodyConfig {
				core: Some(Core {
					shape,
					physics_type: physics_type(HashSet::new()),
				}),
				..default()
			}));

			assert_eq!(sensor, entity.get::<Sensor>());
		}
	}

	mod sub_frames {
		use crate::components::collider::ChildColliderOf;

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
				.spawn(Body(BodyConfig {
					sub_frames: vec![InteractiveFrame::from(shape)],
					..default()
				}))
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
				.spawn(Body(BodyConfig {
					sub_frames: vec![
						InteractiveFrame::from(shape).with_forward_offset(Units::from(11.)),
					],
					..default()
				}))
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
				.spawn(Body(BodyConfig {
					sub_frames: vec![InteractiveFrame::from(shape)],
					..default()
				}))
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
				.spawn(Body(BodyConfig {
					sub_frames: vec![InteractiveFrame::from(shape)],
					..default()
				}))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(Some(&Interactive), child.get::<Interactive>());
		}

		#[test]
		fn insert_child_collider_of() {
			let mut app = setup();
			let shape = ShapeParameters::Sphere {
				radius: Units::from(42.),
			};
			let entity = app
				.world_mut()
				.spawn(Body(BodyConfig {
					sub_frames: vec![InteractiveFrame::from(shape)],
					..default()
				}))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(
				Some(&ChildColliderOf(entity)),
				child.get::<ChildColliderOf>(),
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
				.spawn(Body(BodyConfig {
					sub_frames: vec![InteractiveFrame::from(shape)],
					..default()
				}))
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
				.spawn(Body(BodyConfig {
					sub_frames: vec![InteractiveFrame::from(shape)],
					..default()
				}))
				.id();

			let [child] = assert_children_count!(1, app, entity);
			assert_eq!(Some(&Sensor), child.get::<Sensor>());
		}
	}
}
