use super::{Integrity, Motion, Shape};
use crate::components::{
	ground_target::GroundTarget,
	set_position_and_rotation::SetPositionAndRotation,
	set_to_move_forward::SetVelocityForward,
	when_traveled_insert::WhenTraveled,
	Always,
	Once,
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier3d::prelude::{
	ActiveCollisionTypes,
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
	bundles::{AssetModelBundle, ColliderTransformBundle},
	components::{AssetModel, ColliderRoot},
	errors::{Error, Level},
	traits::{
		handles_destruction::HandlesDestruction,
		handles_interactions::HandlesInteractions,
		prefab::{AfterInstantiation, GetOrCreateAssets, Prefab},
	},
};
use std::f32::consts::PI;

#[derive(Component, Debug, Clone)]
pub struct SkillContact {
	pub shape: Shape,
	pub integrity: Integrity,
	pub motion: Motion,
}

impl SkillContact {
	const SPHERE_MODEL: &str = "models/sphere.glb";

	fn prefab<TInteractions, TLifeCycles>(&self, entity: &mut EntityCommands) -> Result<(), Error>
	where
		TInteractions: HandlesInteractions,
		TLifeCycles: HandlesDestruction,
	{
		self.insert_shape_components(entity)?;
		self.insert_motion_components::<TLifeCycles>(entity);
		self.insert_integrity_components::<TInteractions>(entity);

		Ok(())
	}

	fn insert_shape_components(&self, entity: &mut EntityCommands) -> Result<(), Error> {
		let (model, mut collider, scale) = match &self.shape {
			Shape::Sphere { radius } => (
				AssetModel::path(Self::SPHERE_MODEL),
				Self::sphere_collider()?,
				Vec3::splat(**radius),
			),
			Shape::Custom {
				model,
				collider,
				scale,
			} => (model.clone(), Self::custom_collider(collider), *scale),
		};
		collider.active_collision_types = match &self.motion {
			Motion::HeldBy { .. } => ActiveCollisionTypes::STATIC_STATIC,
			Motion::Stationary { .. } => ActiveCollisionTypes::STATIC_STATIC,
			Motion::Projectile { .. } => ActiveCollisionTypes::default(),
		};

		entity
			.try_insert(SpatialBundle {
				transform: Transform::from_scale(scale),
				..default()
			})
			.with_children(|parent| {
				parent.spawn(AssetModelBundle { model, ..default() });
				parent.spawn((collider, ColliderRoot(parent.parent_entity()), Sensor));
			});

		Ok(())
	}

	fn insert_motion_components<TLifeCycles>(&self, entity: &mut EntityCommands)
	where
		TLifeCycles: HandlesDestruction,
	{
		match self.motion {
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
	}

	fn insert_integrity_components<TInteractions>(&self, entity: &mut EntityCommands)
	where
		TInteractions: HandlesInteractions,
	{
		match &self.integrity {
			Integrity::Solid => {}
			Integrity::Fragile { destroyed_by } => {
				entity.try_insert(TInteractions::is_fragile_when_colliding_with(destroyed_by));
			}
		};
	}

	fn sphere_collider() -> Result<ColliderTransformBundle, Error> {
		let transform = Transform::default().with_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.));
		let ring = Annulus::new(0.9, 1.);
		let torus = Mesh::from(Extrusion::new(ring, 1.));
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

	fn custom_collider(collider: &Collider) -> ColliderTransformBundle {
		ColliderTransformBundle {
			collider: collider.clone(),
			active_events: ActiveEvents::COLLISION_EVENTS,
			..default()
		}
	}
}

impl<TInteractions, TLifeCycles> Prefab<(TInteractions, TLifeCycles)> for SkillContact
where
	TInteractions: HandlesInteractions,
	TLifeCycles: HandlesDestruction,
{
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		_: impl GetOrCreateAssets,
	) -> Result<(), Error>
	where
		TAfterInstantiation: AfterInstantiation,
	{
		self.prefab::<TInteractions, TLifeCycles>(entity)
	}
}

#[cfg(test)]
mod tests {
	use crate::components::when_traveled_insert::InsertAfterDistanceTraveled;

	use super::*;
	use bevy::ecs::system::RunSystemOnce;
	use common::{
		assert_bundle,
		blocker::Blocker,
		bundles::AssetModelBundle,
		components::{AssetModel, ColliderRoot},
		tools::{Units, UnitsPerSecond},
		traits::{clamp_zero_positive::ClampZeroPositive, handles_interactions::BeamParameters},
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
	fn spatial_bundle_circle() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(42.),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(42),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		assert_bundle!(
			SpatialBundle,
			&app,
			app.world().entity(entity),
			With::assert(|transform: &Transform| {
				assert_eq!(&Transform::from_scale(Vec3::splat(42.)), transform);
			})
		);
	}

	#[test]
	fn spatial_bundle_custom_shape() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Custom {
				model: AssetModel::path(""),
				collider: Collider::default(),
				scale: Vec3::new(1., 2., 3.),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(42),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		assert_bundle!(
			SpatialBundle,
			&app,
			app.world().entity(entity),
			With::assert(|transform: &Transform| {
				assert_eq!(&Transform::from_scale(Vec3::new(1., 2., 3.)), transform);
			})
		);
	}

	#[test]
	fn held_components() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(42.),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(11),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
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
		let contact = SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(42.),
			},
			integrity: Integrity::Solid,
			motion: Motion::Stationary {
				caster: Entity::from_raw(42),
				max_cast_range: Units::new(42.),
				target_ray: Ray3d {
					origin: Vec3::new(11., 12., 13.),
					direction: Dir3::from_xyz(2., 1., 4.).unwrap(),
				},
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
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
		let contact = SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(42.),
			},
			integrity: Integrity::Solid,
			motion: Motion::Projectile {
				caster: Entity::from_raw(55),
				spawner: Entity::from_raw(66),
				speed: UnitsPerSecond::new(11.),
				max_range: Units::new(1111.),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
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
		let contact = SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(42.),
			},
			integrity: Integrity::Fragile {
				destroyed_by: vec![Blocker::Physical],
			},
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(42),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
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
	fn shape_sphere() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(42.),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(42),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		let child = children_of(&app, entity)
			.next()
			.expect("no entity children");
		assert_bundle!(
			AssetModelBundle,
			&app,
			child,
			With::assert(|model: &AssetModel| {
				assert_eq!(&AssetModel::path(SkillContact::SPHERE_MODEL), model);
			})
		);
	}

	#[test]
	fn collider_sphere() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(42.),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(42),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		let expected = SkillContact::sphere_collider().unwrap();
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
			// 		collider.as_trimesh().map(|t| t.raw.)
			// 	);
			// }),
			With::assert(|active_events: &ActiveEvents| {
				assert_eq!(&ActiveEvents::COLLISION_EVENTS, active_events);
			})
		);
	}

	#[test]
	fn collider_sphere_sensor() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(42.),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(42),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert!(child.contains::<Sensor>());
	}

	#[test]
	fn collider_sphere_root() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(42.),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(42),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_eq!(Some(&ColliderRoot(entity)), child.get::<ColliderRoot>());
	}

	#[test]
	fn shape_custom() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Custom {
				model: AssetModel::path("custom"),
				collider: Collider::cuboid(1., 2., 3.),
				scale: Vec3::default(),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(42),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		let child = children_of(&app, entity)
			.next()
			.expect("no entity children");
		assert_bundle!(
			AssetModelBundle,
			&app,
			child,
			With::assert(|model: &AssetModel| {
				assert_eq!(&AssetModel::path("custom"), model);
			})
		);
	}

	#[test]
	fn collider_custom() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Custom {
				model: AssetModel::path("custom"),
				collider: Collider::cuboid(1., 2., 3.),
				scale: Vec3::default(),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(42),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_bundle!(
			ColliderTransformBundle,
			&app,
			child,
			With::assert(|collider: &Collider| {
				assert_eq!(
					Collider::cuboid(1., 2., 3.).as_cuboid().map(|c| c.raw),
					collider.as_cuboid().map(|c| c.raw)
				);
			}),
			With::assert(|active_events: &ActiveEvents| {
				assert_eq!(&ActiveEvents::COLLISION_EVENTS, active_events);
			})
		);
	}

	#[test]
	fn collider_custom_sensor() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Custom {
				model: AssetModel::path("custom"),
				collider: Collider::cuboid(1., 2., 3.),
				scale: Vec3::default(),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(42),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert!(child.contains::<Sensor>());
	}

	#[test]
	fn collider_custom_root() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Custom {
				model: AssetModel::path("custom"),
				collider: Collider::cuboid(1., 2., 3.),
				scale: Vec3::default(),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(42),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_eq!(Some(&ColliderRoot(entity)), child.get::<ColliderRoot>());
	}

	#[test]
	fn collision_types_stationary() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(1.),
			},
			integrity: Integrity::Solid,
			motion: Motion::Stationary {
				caster: Entity::from_raw(42),
				max_cast_range: Units::new(100.),
				target_ray: Ray3d::new(Vec3::default(), Vec3::X),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_bundle!(
			ColliderTransformBundle,
			&app,
			child,
			With::assert(|active_collision_events: &ActiveCollisionTypes| {
				assert_eq!(
					&ActiveCollisionTypes::STATIC_STATIC,
					active_collision_events
				);
			})
		);
	}

	#[test]
	fn collision_types_projectile() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(1.),
			},
			integrity: Integrity::Solid,
			motion: Motion::Projectile {
				caster: Entity::from_raw(24),
				spawner: Entity::from_raw(42),
				speed: UnitsPerSecond::new(11.),
				max_range: Units::new(1.),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_bundle!(
			ColliderTransformBundle,
			&app,
			child,
			With::assert(|active_collision_events: &ActiveCollisionTypes| {
				assert_eq!(&ActiveCollisionTypes::default(), active_collision_events);
			})
		);
	}

	#[test]
	fn collision_types_held() {
		let (mut app, entity) = setup();
		let contact = SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(1.),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy {
				spawner: Entity::from_raw(42),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			contact.prefab::<_Interactions, _LifeCycles>(entity)
		}));

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_bundle!(
			ColliderTransformBundle,
			&app,
			child,
			With::assert(|active_collision_events: &ActiveCollisionTypes| {
				assert_eq!(
					&ActiveCollisionTypes::STATIC_STATIC,
					active_collision_events
				);
			})
		);
	}
}
