pub mod skill_contact;
pub mod skill_projection;

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier3d::prelude::{
	ActiveEvents,
	Ccd,
	Collider,
	ComputedColliderShape,
	GravityScale,
	RigidBody,
	Sensor,
	Velocity,
};
use common::{
	blocker::Blocker,
	bundles::{AssetModelBundle, ColliderTransformBundle},
	components::{AssetModel, ColliderRoot, Outdated},
	errors::{Error, Level},
	resources::ColliderInfo,
	tools::{Units, UnitsPerSecond},
	traits::{handles_destruction::HandlesDestruction, handles_interactions::HandlesInteractions},
};
use std::f32::consts::PI;

use super::{
	ground_target::GroundTarget,
	set_position_and_rotation::SetPositionAndRotation,
	set_to_move_forward::SetVelocityForward,
	when_traveled_insert::WhenTraveled,
	Always,
	Once,
};

#[derive(Debug, Clone)]
pub enum Shape {
	Sphere {
		radius: Units,
		hollow_collider: bool,
	},
	Custom {
		model: AssetModel,
		collider: Collider,
		scale: Vec3,
	},
}

impl Shape {
	const SPHERE_MODEL: &str = "models/sphere.glb";

	fn prefab(&self, entity: &mut EntityCommands, offset: Vec3) -> Result<(), Error> {
		let (model, collider) = match self {
			Shape::Sphere {
				radius,
				hollow_collider,
			} => (
				AssetModelBundle {
					model: AssetModel::path(Self::SPHERE_MODEL),
					transform: Transform::from_scale(Vec3::splat(**radius * 2.)),
					..default()
				},
				match hollow_collider {
					true => Self::ring_collider(**radius)?,
					false => Self::sphere_collider(**radius),
				},
			),
			Shape::Custom {
				model,
				collider,
				scale,
			} => (
				AssetModelBundle {
					model: model.clone(),
					transform: Transform::from_scale(*scale),
					..default()
				},
				Self::custom_collider(collider, *scale),
			),
		};

		entity
			.try_insert(SpatialBundle {
				transform: Transform::from_translation(offset),
				..default()
			})
			.with_children(|parent| {
				parent.spawn(model);
				parent.spawn((collider, ColliderRoot(parent.parent_entity()), Sensor));
			});

		Ok(())
	}

	fn sphere_collider(radius: f32) -> ColliderTransformBundle {
		ColliderTransformBundle {
			collider: Collider::ball(radius),
			active_events: ActiveEvents::COLLISION_EVENTS,
			transform: Transform::default(),
			..default()
		}
	}

	fn ring_collider(radius: f32) -> Result<ColliderTransformBundle, Error> {
		let transform = Transform::default().with_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.));
		let ring = Annulus::new(radius * 0.9, radius);
		let torus = Mesh::from(Extrusion::new(ring, 3.));
		let collider = Collider::from_bevy_mesh(&torus, &ComputedColliderShape::TriMesh);

		let Some(collider) = collider else {
			return Err(Error {
				msg: "Cannot create spherical contact collider".to_owned(),
				lvl: Level::Error,
			});
		};

		Ok(ColliderTransformBundle {
			transform,
			collider,
			active_events: ActiveEvents::COLLISION_EVENTS,
			..default()
		})
	}

	fn custom_collider(collider: &Collider, scale: Vec3) -> ColliderTransformBundle {
		ColliderTransformBundle {
			collider: collider.clone(),
			active_events: ActiveEvents::COLLISION_EVENTS,
			transform: Transform::from_scale(scale),
			..default()
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum Integrity {
	Solid,
	Fragile { destroyed_by: Vec<Blocker> },
}

impl Integrity {
	fn prefab<TInteractions>(&self, entity: &mut EntityCommands) -> Result<(), Error>
	where
		TInteractions: HandlesInteractions,
	{
		match self {
			Integrity::Solid => {}
			Integrity::Fragile { destroyed_by } => {
				entity.try_insert(TInteractions::is_fragile_when_colliding_with(destroyed_by));
			}
		};

		Ok(())
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum Motion {
	HeldBy {
		spawner: Entity,
	},
	Stationary {
		caster: Entity,
		max_cast_range: Units,
		target_ray: Ray3d,
	},
	Projectile {
		caster: Entity,
		spawner: Entity,
		speed: UnitsPerSecond,
		max_range: Units,
	},
}

impl Motion {
	fn prefab<TLifeCycles>(&self, entity: &mut EntityCommands) -> Result<(), Error>
	where
		TLifeCycles: HandlesDestruction,
	{
		match *self {
			Motion::HeldBy { spawner } => {
				entity.try_insert((
					RigidBody::Fixed,
					SetPositionAndRotation::<Always>::to(spawner),
				));
			}
			Motion::Stationary {
				caster,
				max_cast_range,
				target_ray,
			} => {
				entity.try_insert((
					RigidBody::Fixed,
					GroundTarget {
						caster,
						max_cast_range,
						target_ray,
					},
				));
			}
			Motion::Projectile {
				caster,
				spawner,
				speed,
				max_range,
			} => {
				entity.try_insert((
					RigidBody::Dynamic,
					GravityScale(0.),
					Ccd::enabled(),
					SetPositionAndRotation::<Once>::to(spawner),
					SetVelocityForward {
						rotation: caster,
						speed,
					},
					WhenTraveled::via::<Velocity>()
						.distance(max_range)
						.insert::<TLifeCycles::TDestroy>(),
				));
			}
		}
		Ok(())
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Offset(pub Vec3);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SelectInfo<T> {
	pub ray: Ray3d,
	pub collision_info: Option<ColliderInfo<T>>,
}

impl<T> Default for SelectInfo<T> {
	fn default() -> Self {
		Self {
			ray: Ray3d {
				origin: Vec3::ZERO,
				direction: Dir3::NEG_Z,
			},
			collision_info: None,
		}
	}
}

pub type SkillTarget = SelectInfo<Outdated<GlobalTransform>>;

impl From<Ray3d> for SkillTarget {
	fn from(ray: Ray3d) -> Self {
		Self { ray, ..default() }
	}
}

impl SkillTarget {
	pub fn with_ray(self, ray: Ray3d) -> Self {
		Self {
			ray,
			collision_info: self.collision_info,
		}
	}

	pub fn with_collision_info(
		self,
		collision_info: Option<ColliderInfo<Outdated<GlobalTransform>>>,
	) -> Self {
		Self {
			ray: self.ray,
			collision_info,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::when_traveled_insert::InsertAfterDistanceTraveled;
	use bevy::ecs::system::RunSystemOnce;
	use bevy_rapier3d::prelude::ActiveCollisionTypes;
	use common::{
		assert_bundle,
		blocker::Blocker,
		bundles::AssetModelBundle,
		components::{AssetModel, ColliderRoot},
		tools::{Units, UnitsPerSecond},
		traits::{
			clamp_zero_positive::ClampZeroPositive,
			handles_destruction::HandlesDestruction,
			handles_interactions::{BeamParameters, HandlesInteractions},
		},
	};

	struct _Interactions;

	impl HandlesInteractions for _Interactions {
		fn is_fragile_when_colliding_with(blockers: &[Blocker]) -> impl Bundle {
			_IsFragile(Vec::from(blockers))
		}

		fn is_ray_interrupted_by(_: &[Blocker]) -> impl Bundle {}

		fn beam_from<T>(_: &T) -> impl Bundle
		where
			T: BeamParameters,
		{
		}
	}

	struct _LifeCycles;

	impl HandlesDestruction for _LifeCycles {
		type TDestroy = _Destroy;
	}

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Destroy;

	#[derive(Component, Debug, PartialEq)]
	struct _IsFragile(Vec<Blocker>);

	fn test_system<T>(
		exec: impl Fn(&mut EntityCommands) -> T,
	) -> impl Fn(Commands, Query<Entity>) -> T {
		move |mut commands, query| {
			let entity = query.single();
			let mut entity = commands.entity(entity);

			exec(&mut entity)
		}
	}

	fn setup() -> (App, Entity) {
		let mut app = App::new();
		let entity = app.world_mut().spawn_empty().id();

		(app, entity)
	}

	#[test]
	fn held_components() {
		let (mut app, entity) = setup();
		let motion = Motion::HeldBy {
			spawner: Entity::from_raw(11),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			motion.prefab::<_LifeCycles>(entity)
		}));

		assert_eq!(
			(
				Some(&RigidBody::Fixed),
				None,
				None,
				None,
				Some(&SetPositionAndRotation::<Always>::to(Entity::from_raw(11))),
				None,
				None,
				None,
			),
			(
				app.world().entity(entity).get::<RigidBody>(),
				app.world().entity(entity).get::<GravityScale>(),
				app.world().entity(entity).get::<Ccd>(),
				app.world().entity(entity).get::<GroundTarget>(),
				app.world()
					.entity(entity)
					.get::<SetPositionAndRotation<Always>>(),
				app.world()
					.entity(entity)
					.get::<SetPositionAndRotation<Once>>(),
				app.world().entity(entity).get::<SetVelocityForward>(),
				app.world()
					.entity(entity)
					.get::<InsertAfterDistanceTraveled<_Destroy, Velocity>>(),
			)
		);
	}

	#[test]
	fn stationary_components() {
		let (mut app, entity) = setup();
		let motion = Motion::Stationary {
			caster: Entity::from_raw(42),
			max_cast_range: Units::new(42.),
			target_ray: Ray3d {
				origin: Vec3::new(11., 12., 13.),
				direction: Dir3::from_xyz(2., 1., 4.).unwrap(),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			motion.prefab::<_LifeCycles>(entity)
		}));

		assert_eq!(
			(
				Some(&RigidBody::Fixed),
				None,
				None,
				Some(&GroundTarget {
					caster: Entity::from_raw(42),
					max_cast_range: Units::new(42.),
					target_ray: Ray3d {
						origin: Vec3::new(11., 12., 13.),
						direction: Dir3::from_xyz(2., 1., 4.).unwrap(),
					},
				}),
				None,
				None,
				None,
				None,
			),
			(
				app.world().entity(entity).get::<RigidBody>(),
				app.world().entity(entity).get::<GravityScale>(),
				app.world().entity(entity).get::<Ccd>(),
				app.world().entity(entity).get::<GroundTarget>(),
				app.world()
					.entity(entity)
					.get::<SetPositionAndRotation<Always>>(),
				app.world()
					.entity(entity)
					.get::<SetPositionAndRotation<Once>>(),
				app.world().entity(entity).get::<SetVelocityForward>(),
				app.world()
					.entity(entity)
					.get::<InsertAfterDistanceTraveled<_Destroy, Velocity>>(),
			)
		);
	}

	#[test]
	fn projectile_components() {
		let (mut app, entity) = setup();
		let motion = Motion::Projectile {
			caster: Entity::from_raw(55),
			spawner: Entity::from_raw(66),
			speed: UnitsPerSecond::new(11.),
			max_range: Units::new(1111.),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			motion.prefab::<_LifeCycles>(entity)
		}));

		assert_eq!(
			(
				Some(&RigidBody::Dynamic),
				Some(&GravityScale(0.)),
				Some(&Ccd::enabled()),
				None,
				None,
				Some(&SetPositionAndRotation::<Once>::to(Entity::from_raw(66))),
				Some(&SetVelocityForward {
					rotation: Entity::from_raw(55),
					speed: UnitsPerSecond::new(11.),
				}),
				Some(
					&WhenTraveled::via::<Velocity>()
						.distance(Units::new(1111.))
						.insert::<_Destroy>()
				),
			),
			(
				app.world().entity(entity).get::<RigidBody>(),
				app.world().entity(entity).get::<GravityScale>(),
				app.world().entity(entity).get::<Ccd>(),
				app.world().entity(entity).get::<GroundTarget>(),
				app.world()
					.entity(entity)
					.get::<SetPositionAndRotation<Always>>(),
				app.world()
					.entity(entity)
					.get::<SetPositionAndRotation<Once>>(),
				app.world().entity(entity).get::<SetVelocityForward>(),
				app.world()
					.entity(entity)
					.get::<InsertAfterDistanceTraveled<_Destroy, Velocity>>(),
			)
		);
	}

	#[test]
	fn fragile_components() {
		let (mut app, entity) = setup();
		let integrity = Integrity::Fragile {
			destroyed_by: vec![Blocker::Physical],
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			integrity.prefab::<_Interactions>(entity)
		}));

		assert_eq!(
			Some(&_IsFragile(vec![Blocker::Physical])),
			app.world().entity(entity).get::<_IsFragile>(),
		);
	}

	fn children_of(app: &App, entity: Entity) -> impl Iterator<Item = EntityRef> {
		app.world().iter_entities().filter(move |e| {
			e.get::<Parent>()
				.map(|p| p.get() == entity)
				.unwrap_or(false)
		})
	}

	#[test]
	fn spatial_bundle_sphere() {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: false,
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab(entity, Vec3::new(1., 2., 3.))
		}));

		assert_bundle!(
			SpatialBundle,
			&app,
			app.world().entity(entity),
			With::assert(|transform: &Transform| {
				assert_eq!(&Transform::from_xyz(1., 2., 3.), transform);
			})
		);
	}

	#[test]
	fn spatial_bundle_custom_shape() {
		let (mut app, entity) = setup();
		let shape = Shape::Custom {
			model: AssetModel::path(""),
			collider: Collider::default(),
			scale: Vec3::new(1., 2., 3.),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab(entity, Vec3::new(1., 2., 3.))
		}));

		assert_bundle!(
			SpatialBundle,
			&app,
			app.world().entity(entity),
			With::assert(|transform: &Transform| {
				assert_eq!(&Transform::from_xyz(1., 2., 3.), transform);
			})
		);
	}

	#[test]
	fn shape_sphere() {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: false,
		};

		_ = app
			.world_mut()
			.run_system_once(test_system(move |entity| shape.prefab(entity, Vec3::ZERO)));

		let child = children_of(&app, entity)
			.next()
			.expect("no entity children");
		assert_bundle!(
			AssetModelBundle,
			&app,
			child,
			With::assert(|transform: &Transform| {
				assert_eq!(&Transform::from_scale(Vec3::splat(42. * 2.)), transform);
			}),
			With::assert(|model: &AssetModel| {
				assert_eq!(&AssetModel::path(Shape::SPHERE_MODEL), model);
			})
		);
	}

	#[test]
	fn collider_sphere_sphere() {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: false,
		};

		_ = app
			.world_mut()
			.run_system_once(test_system(move |entity| shape.prefab(entity, Vec3::ZERO)));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_bundle!(
			ColliderTransformBundle,
			&app,
			child,
			With::assert(|transform: &Transform| {
				assert_eq!(&Transform::default(), transform);
			}),
			With::assert(|collider: &Collider| {
				assert_eq!(
					Collider::ball(42.).as_ball().map(|b| b.raw),
					collider.as_ball().map(|b| b.raw),
				);
			}),
			With::assert(|active_events: &ActiveEvents| {
				assert_eq!(&ActiveEvents::COLLISION_EVENTS, active_events);
			}),
			With::assert(|active_collision_events: &ActiveCollisionTypes| {
				assert_eq!(&ActiveCollisionTypes::default(), active_collision_events);
			})
		);
	}

	#[test]
	fn collider_sphere_hollow_as_ring() {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: true,
		};

		_ = app
			.world_mut()
			.run_system_once(test_system(move |entity| shape.prefab(entity, Vec3::ZERO)));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		let expected = Shape::ring_collider(42.).unwrap();
		assert_bundle!(
			ColliderTransformBundle,
			&app,
			child,
			With::assert(|transform: &Transform| {
				assert_eq!(&expected.transform, transform);
			}),
			// FIXME: The trimesh data is not directly comparable.
			// The following assertion would ideally verify equality,
			// but `TriMesh` (t.raw) does not implement `PartialEq`.
			// With::assert(|collider: &Collider| {
			// 	assert_eq!(
			// 		expected.collider.as_trimesh().map(|t| t.raw),
			// 		collider.as_trimesh().map(|t| t.raw),
			// 	);
			// }),
			With::assert(|active_events: &ActiveEvents| {
				assert_eq!(&ActiveEvents::COLLISION_EVENTS, active_events);
			}),
			With::assert(|active_collision_events: &ActiveCollisionTypes| {
				assert_eq!(&ActiveCollisionTypes::default(), active_collision_events);
			})
		);
	}

	#[test]
	fn collider_sphere_sensor() {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: false,
		};

		_ = app
			.world_mut()
			.run_system_once(test_system(move |entity| shape.prefab(entity, Vec3::ZERO)));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert!(child.contains::<Sensor>());
	}

	#[test]
	fn collider_sphere_root() {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: false,
		};

		_ = app
			.world_mut()
			.run_system_once(test_system(move |entity| shape.prefab(entity, Vec3::ZERO)));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_eq!(Some(&ColliderRoot(entity)), child.get::<ColliderRoot>());
	}

	#[test]
	fn shape_custom() {
		let (mut app, entity) = setup();
		let shape = Shape::Custom {
			model: AssetModel::path("custom"),
			collider: Collider::cuboid(1., 2., 3.),
			scale: Vec3::new(3., 2., 1.),
		};

		_ = app
			.world_mut()
			.run_system_once(test_system(move |entity| shape.prefab(entity, Vec3::ZERO)));

		let child = children_of(&app, entity)
			.next()
			.expect("no entity children");
		assert_bundle!(
			AssetModelBundle,
			&app,
			child,
			With::assert(|transform: &Transform| {
				assert_eq!(&Transform::from_scale(Vec3::new(3., 2., 1.)), transform);
			}),
			With::assert(|model: &AssetModel| {
				assert_eq!(&AssetModel::path("custom"), model);
			})
		);
	}

	#[test]
	fn collider_custom() {
		let (mut app, entity) = setup();
		let shape = Shape::Custom {
			model: AssetModel::path("custom"),
			collider: Collider::cuboid(1., 2., 3.),
			scale: Vec3::new(3., 2., 1.),
		};

		_ = app
			.world_mut()
			.run_system_once(test_system(move |entity| shape.prefab(entity, Vec3::ZERO)));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_bundle!(
			ColliderTransformBundle,
			&app,
			child,
			With::assert(|transform: &Transform| {
				assert_eq!(&Transform::from_scale(Vec3::new(3., 2., 1.)), transform);
			}),
			With::assert(|collider: &Collider| {
				assert_eq!(
					Collider::cuboid(1., 2., 3.).as_cuboid().map(|c| c.raw),
					collider.as_cuboid().map(|c| c.raw)
				);
			}),
			With::assert(|active_events: &ActiveEvents| {
				assert_eq!(&ActiveEvents::COLLISION_EVENTS, active_events);
			}),
			With::assert(|active_collision_events: &ActiveCollisionTypes| {
				assert_eq!(&ActiveCollisionTypes::default(), active_collision_events);
			})
		);
	}

	#[test]
	fn collider_custom_sensor() {
		let (mut app, entity) = setup();
		let shape = Shape::Custom {
			model: AssetModel::path("custom"),
			collider: Collider::cuboid(1., 2., 3.),
			scale: Vec3::default(),
		};

		_ = app
			.world_mut()
			.run_system_once(test_system(move |entity| shape.prefab(entity, Vec3::ZERO)));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert!(child.contains::<Sensor>());
	}

	#[test]
	fn collider_custom_root() {
		let (mut app, entity) = setup();
		let shape = Shape::Custom {
			model: AssetModel::path("custom"),
			collider: Collider::cuboid(1., 2., 3.),
			scale: Vec3::default(),
		};

		_ = app
			.world_mut()
			.run_system_once(test_system(move |entity| shape.prefab(entity, Vec3::ZERO)));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_eq!(Some(&ColliderRoot(entity)), child.get::<ColliderRoot>());
	}
}
