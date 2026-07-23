use crate::components::{
	blocker_types::BlockerTypes,
	collider::{
		AGENTS_GROUP,
		ColliderOf,
		ColliderShape,
		INTERACTIVE_GROUP,
		RAY_GROUP,
		SKILLS_GROUP,
		TERRAIN_GROUP,
	},
	collision_domains::{Interactive, Physical},
	motion_controller::MotionCollider,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use bevy_rapier3d::prelude::*;
use common::{
	errors::Unreachable,
	traits::{
		handles_physics::physical_bodies::{Blocker, BodyConfig, PhysicsType, Shape},
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use std::collections::HashSet;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub struct Body(pub(crate) BodyConfig);

impl Body {
	fn agent(shape: Shape, blockers: HashSet<Blocker>) -> impl Bundle {
		(MotionCollider { shape }, BlockerTypes(blockers))
	}

	fn terrain(shape: Shape, blockers: HashSet<Blocker>) -> impl Bundle {
		(
			ColliderShape::from(shape),
			BlockerTypes(blockers),
			Physical::Contact,
			RigidBody::Fixed,
			CollisionGroups {
				memberships: TERRAIN_GROUP,
				filters: SKILLS_GROUP | AGENTS_GROUP | RAY_GROUP,
			},
		)
	}

	fn interactive_of(entity: Entity) -> impl Bundle {
		(
			Interactive,
			Sensor,
			ColliderOf(entity),
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
		_: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), Self::TError> {
		let Self(BodyConfig { core, sub_frames }) = self;

		if let Some(core) = core {
			match core.physics_type {
				PhysicsType::Agent(ref blockers) => {
					entity.try_insert(Self::agent(core.shape, blockers.clone()));
				}
				PhysicsType::Terrain(ref blockers) => {
					entity.try_insert(Self::terrain(core.shape, blockers.clone()));
				}
			};
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

		mod agent {
			use super::*;

			#[test]
			fn insert_motion_collider() {
				let mut app = setup();
				let shape = Shape::Parameters(ShapeParameters::Sphere {
					radius: Units::from(42.),
				});

				let entity = app
					.world_mut()
					.spawn(Body(BodyConfig {
						core: Some(Core {
							shape,
							physics_type: PhysicsType::Agent(HashSet::from([])),
						}),
						..default()
					}))
					.id();

				assert_eq!(
					Some(&MotionCollider {
						shape: Shape::Parameters(ShapeParameters::Sphere {
							radius: Units::from(42.),
						}),
					}),
					app.world().entity(entity).get::<MotionCollider>(),
				);
			}

			#[test]
			fn insert_control_blockers() {
				let mut app = setup();
				let shape = Shape::Parameters(ShapeParameters::Sphere {
					radius: Units::from(42.),
				});

				let entity = app
					.world_mut()
					.spawn(Body(BodyConfig {
						core: Some(Core {
							shape,
							physics_type: PhysicsType::Agent(HashSet::from([
								Blocker::Character,
								Blocker::Force,
							])),
						}),
						..default()
					}))
					.id();

				assert_eq!(
					Some(&BlockerTypes(HashSet::from([
						Blocker::Character,
						Blocker::Force,
					]))),
					app.world().entity(entity).get::<BlockerTypes>(),
				);
			}
		}

		mod terrain {
			use super::*;

			#[test]
			fn insert_collider() {
				let mut app = setup();
				let shape = Shape::Parameters(ShapeParameters::Sphere {
					radius: Units::from(42.),
				});

				let entity = app
					.world_mut()
					.spawn(Body(BodyConfig {
						core: Some(Core {
							shape,
							physics_type: PhysicsType::Terrain(HashSet::new()),
						}),
						..default()
					}))
					.id();

				assert_eq!(
					Some(&ColliderShape::from(shape)),
					app.world().entity(entity).get::<ColliderShape>(),
				);
			}

			#[test]
			fn insert_physical_contact() {
				let mut app = setup();
				let shape = Shape::Parameters(ShapeParameters::Sphere {
					radius: Units::from(42.),
				});

				let entity = app.world_mut().spawn(Body(BodyConfig {
					core: Some(Core {
						shape,
						physics_type: PhysicsType::Terrain(HashSet::default()),
					}),
					..default()
				}));

				assert_eq!(Some(&Physical::Contact), entity.get::<Physical>());
			}

			#[test]
			fn insert_physics_constraints() {
				let mut app = setup();
				let shape = Shape::Parameters(ShapeParameters::Sphere {
					radius: Units::from(42.),
				});

				let entity = app.world_mut().spawn(Body(BodyConfig {
					core: Some(Core {
						shape,
						physics_type: PhysicsType::Terrain(HashSet::new()),
					}),
					..default()
				}));

				assert_eq!(
					(Some(&RigidBody::Fixed), false, false, None, None),
					(
						entity.get::<RigidBody>(),
						entity.contains::<KinematicCharacterController>(),
						entity.contains::<CollidingEntities>(),
						entity.get::<ActiveEvents>(),
						entity.get::<ActiveCollisionTypes>(),
					)
				);
			}

			#[test]
			fn insert_blocker_types() {
				let mut app = setup();
				let shape = Shape::Parameters(ShapeParameters::Sphere {
					radius: Units::from(42.),
				});

				let entity = app.world_mut().spawn(Body(BodyConfig {
					core: Some(Core {
						shape,
						physics_type: PhysicsType::Terrain(HashSet::from([
							Blocker::Force,
							Blocker::Physical,
						])),
					}),
					..default()
				}));

				assert_eq!(
					Some(BlockerTypes(HashSet::from([
						Blocker::Force,
						Blocker::Physical
					]))),
					entity.get::<BlockerTypes>().cloned(),
				);
			}

			#[test]
			fn insert_collision_groups() {
				let mut app = setup();
				let shape = Shape::Parameters(ShapeParameters::Sphere {
					radius: Units::from(42.),
				});

				let entity = app.world_mut().spawn(Body(BodyConfig {
					core: Some(Core {
						shape,
						physics_type: PhysicsType::Terrain(HashSet::new()),
					}),
					..default()
				}));

				assert_eq!(
					Some(&CollisionGroups {
						memberships: TERRAIN_GROUP,
						filters: SKILLS_GROUP | AGENTS_GROUP | RAY_GROUP
					}),
					entity.get::<CollisionGroups>(),
				);
			}
		}
	}

	mod sub_frames {
		use super::*;
		use crate::components::collider::ColliderOf;
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
			assert_eq!(Some(&ColliderOf(entity)), child.get::<ColliderOf>(),);
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
