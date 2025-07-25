use crate::components::{Attack, Chase};
use bevy::{math::InvalidDirectionError, prelude::*};
use bevy_rapier3d::plugin::ReadRapierContext;
use common::{
	components::{
		collider_relationship::ColliderOfInteractionTarget,
		ground_offset::GroundOffset,
		persistent_entity::PersistentEntity,
	},
	errors::Error,
	resources::persistent_entities::PersistentEntities,
	tools::{aggro_range::AggroRange, attack_range::AttackRange},
	traits::{
		accessors::get::Getter,
		cast_ray::{
			CastRay,
			GetRayCaster,
			read_rapier_context::{ExcludeRigidBody, NoSensors},
		},
		handles_enemies::EnemyTarget,
	},
};

impl<T> SelectBehavior for T {}

pub(crate) trait SelectBehavior {
	fn select_behavior<TPlayer>(
		rapier: ReadRapierContext,
		commands: Commands,
		agents: Query<(
			&PersistentEntity,
			&GlobalTransform,
			Option<&GroundOffset>,
			&Self,
		)>,
		players: Query<(&PersistentEntity, &GlobalTransform, Option<&GroundOffset>), With<TPlayer>>,
		all: Query<(&PersistentEntity, &GlobalTransform, Option<&GroundOffset>)>,
		colliders: Query<&ColliderOfInteractionTarget>,
		persistent_entities: ResMut<PersistentEntities>,
	) -> Result<(), BehaviorError>
	where
		Self: Component + Sized + Getter<AggroRange> + Getter<AttackRange> + Getter<EnemyTarget>,
		TPlayer: Component,
	{
		select_behavior(
			rapier,
			commands,
			agents,
			players,
			all,
			colliders,
			persistent_entities,
		)
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum BehaviorError<TNoRayCaster = BevyError> {
	NoRayCaster(TNoRayCaster),
	InvalidDirectionErrors(Vec<InvalidDirectionError>),
}

impl From<BehaviorError> for Error {
	fn from(value: BehaviorError) -> Self {
		match value {
			BehaviorError::NoRayCaster(bevy_error) => Error::from(bevy_error),
			BehaviorError::InvalidDirectionErrors(invalid_direction_errors) => Error::Multiple(
				invalid_direction_errors
					.into_iter()
					.map(Error::from)
					.collect(),
			),
		}
	}
}

enum Behavior {
	Attack,
	Chase,
	Idle,
}

fn select_behavior<TAgent, TPlayer, TGetRayCaster>(
	ray_caster_source: TGetRayCaster,
	mut commands: Commands,
	agents: Query<(
		&PersistentEntity,
		&GlobalTransform,
		Option<&GroundOffset>,
		&TAgent,
	)>,
	players: Query<(&PersistentEntity, &GlobalTransform, Option<&GroundOffset>), With<TPlayer>>,
	all: Query<(&PersistentEntity, &GlobalTransform, Option<&GroundOffset>)>,
	colliders: Query<&ColliderOfInteractionTarget>,
	mut persistent_entities: ResMut<PersistentEntities>,
) -> Result<(), BehaviorError<TGetRayCaster::TError>>
where
	TAgent: Component + Sized + Getter<AggroRange> + Getter<AttackRange> + Getter<EnemyTarget>,
	TPlayer: Component,
	TGetRayCaster: GetRayCaster<(Ray3d, NoSensors, ExcludeRigidBody)>,
{
	let ray_caster = match ray_caster_source.get_ray_caster() {
		Ok(ray_caster) => ray_caster,
		Err(error) => return Err(BehaviorError::NoRayCaster(error)),
	};
	let mut invalid_directions = vec![];

	for (persistent_agent_entity, transform, ground_offset, agent) in &agents {
		let target = match agent.get() {
			EnemyTarget::Player => players.single().ok(),
			EnemyTarget::Entity(persistent_entity) => persistent_entities
				.get_entity(&persistent_entity)
				.and_then(|entity| all.get(entity).ok()),
		};
		let Some((persistent_target, target_transform, target_ground_offset)) = target else {
			continue;
		};
		let Some(target_entity) = persistent_entities.get_entity(persistent_target) else {
			continue;
		};
		let Some(agent_entity) = persistent_entities.get_entity(persistent_agent_entity) else {
			continue;
		};
		let translation = match ground_offset {
			Some(GroundOffset(ground_offset)) => transform.translation() + ground_offset,
			None => transform.translation(),
		};
		let target_translation = match target_ground_offset {
			Some(GroundOffset(ground_offset)) => target_transform.translation() + ground_offset,
			None => target_transform.translation(),
		};
		let strategy = get_strategy(
			agent_entity,
			agent,
			translation,
			target_entity,
			target_translation,
			&ray_caster,
			&colliders,
		);
		let Ok(mut agent_entity) = commands.get_entity(agent_entity) else {
			continue;
		};

		match strategy {
			Err(invalid_direction) => invalid_directions.push(invalid_direction),
			Ok(Behavior::Attack) => {
				agent_entity.try_insert(Attack(*persistent_target));
				agent_entity.remove::<Chase>();
			}
			Ok(Behavior::Chase) => {
				agent_entity.try_insert(Chase(*persistent_target));
				agent_entity.remove::<Attack>();
			}
			Ok(Behavior::Idle) => {
				agent_entity.remove::<Chase>();
				agent_entity.remove::<Attack>();
			}
		}
	}

	if !invalid_directions.is_empty() {
		return Err(BehaviorError::InvalidDirectionErrors(invalid_directions));
	}

	Ok(())
}

fn get_strategy<TAgent, TCaster>(
	enemy: Entity,
	enemy_agent: &TAgent,
	enemy_translation: Vec3,
	target: Entity,
	target_translation: Vec3,
	ray_caster: &TCaster,
	colliders: &Query<&ColliderOfInteractionTarget>,
) -> Result<Behavior, InvalidDirectionError>
where
	TAgent: Getter<AggroRange> + Getter<AttackRange>,
	TCaster: CastRay<(Ray3d, NoSensors, ExcludeRigidBody)>,
{
	let direction = target_translation - enemy_translation;
	let distance = direction.length();

	if distance > aggro_range(enemy_agent) {
		return Ok(Behavior::Idle);
	}
	if distance > attack_range(enemy_agent) {
		return Ok(Behavior::Chase);
	}

	let ray = Ray3d {
		origin: enemy_translation,
		direction: Dir3::new(direction)?,
	};

	match ray_caster.cast_ray(&(ray, NoSensors, ExcludeRigidBody(enemy))) {
		Some((hit, ..)) if hit_target(target, hit, colliders) => Ok(Behavior::Attack),
		_ => Ok(Behavior::Chase),
	}
}

fn aggro_range<TAgent>(agent: &TAgent) -> f32
where
	TAgent: Getter<AggroRange>,
{
	**agent.get()
}

fn attack_range<TAgent>(agent: &TAgent) -> f32
where
	TAgent: Getter<AttackRange>,
{
	**agent.get()
}

fn hit_target(
	target: Entity,
	hit: Entity,
	colliders: &Query<&ColliderOfInteractionTarget>,
) -> bool {
	if hit == target {
		return true;
	}

	colliders
		.get(hit)
		.map(|collider| collider.target() == target)
		.unwrap_or(false)
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		tools::Units,
		traits::{
			cast_ray::TimeOfImpact,
			clamp_zero_positive::ClampZeroPositive,
			register_persistent_entities::RegisterPersistentEntities,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::sync::LazyLock;
	use testing::{NestedMocks, SingleThreadedApp};

	static ENEMY: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[derive(Component)]
	#[require(PersistentEntity = *ENEMY)]
	struct _Enemy {
		aggro_range: AggroRange,
		attack_range: AttackRange,
		target: EnemyTarget,
	}

	impl Getter<AggroRange> for _Enemy {
		fn get(&self) -> AggroRange {
			self.aggro_range
		}
	}

	impl Getter<AttackRange> for _Enemy {
		fn get(&self) -> AttackRange {
			self.attack_range
		}
	}

	impl Getter<EnemyTarget> for _Enemy {
		fn get(&self) -> EnemyTarget {
			self.target
		}
	}

	static PLAYER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[derive(Component)]
	#[require(PersistentEntity = *PLAYER)]
	struct _Player;

	static TARGET: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[derive(Component)]
	#[require(PersistentEntity = *TARGET)]
	struct _Target;

	#[derive(NestedMocks)]
	pub struct _GetRayCaster {
		mock: Mock_GetRayCaster,
	}

	impl _GetRayCaster {
		fn with_no_hit() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_get_ray_caster().returning(|| {
					Ok(_RayCaster::new().with_mock(|mock| {
						mock.expect_cast_ray().return_const(None);
					}))
				});
			})
		}
	}

	#[derive(Debug, PartialEq)]
	pub enum _ContextQueryError {}

	#[automock]
	impl GetRayCaster<(Ray3d, NoSensors, ExcludeRigidBody)> for _GetRayCaster {
		type TError = _ContextQueryError;
		type TRayCaster<'a>
			= _RayCaster
		where
			Self: 'a;

		fn get_ray_caster(&self) -> Result<Self::TRayCaster<'_>, Self::TError> {
			self.mock.get_ray_caster()
		}
	}

	#[derive(NestedMocks)]
	pub struct _RayCaster {
		mock: Mock_RayCaster,
	}

	#[automock]
	impl CastRay<(Ray3d, NoSensors, ExcludeRigidBody)> for _RayCaster {
		fn cast_ray(
			&self,
			ray_data: &(Ray3d, NoSensors, ExcludeRigidBody),
		) -> Option<(Entity, TimeOfImpact)> {
			self.mock.cast_ray(ray_data)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();

		app
	}

	#[test]
	fn chase_player() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Player));
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(2., 0., 1.),
				Attack(*PLAYER),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Player,
				},
			))
			.id();

		_ = app.world_mut().run_system_once_with(
			select_behavior::<_Enemy, _Player, In<_GetRayCaster>>,
			_GetRayCaster::with_no_hit(),
		)?;

		let enemy = app.world().entity(enemy);
		assert_eq!(
			(Some(&Chase(*PLAYER)), None),
			(enemy.get::<Chase>(), enemy.get::<Attack>())
		);
		Ok(())
	}

	#[test]
	fn do_nothing_when_out_of_aggro_range_of_player() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Player));
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(3., 0., 3.),
				Chase(*PLAYER),
				Attack(*PLAYER),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Player,
				},
			))
			.id();

		_ = app.world_mut().run_system_once_with(
			select_behavior::<_Enemy, _Player, In<_GetRayCaster>>,
			_GetRayCaster::with_no_hit(),
		)?;

		let enemy = app.world().entity(enemy);
		assert_eq!((None, None), (enemy.get::<Chase>(), enemy.get::<Attack>()));
		Ok(())
	}

	#[test]
	fn chase_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Target));
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(2., 0., 1.),
				Attack(*TARGET),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Entity(*TARGET),
				},
			))
			.id();

		_ = app.world_mut().run_system_once_with(
			select_behavior::<_Enemy, _Player, In<_GetRayCaster>>,
			_GetRayCaster::with_no_hit(),
		)?;

		let enemy = app.world().entity(enemy);
		assert_eq!(
			(Some(&Chase(*TARGET)), None),
			(enemy.get::<Chase>(), enemy.get::<Attack>())
		);
		Ok(())
	}

	#[test]
	fn do_nothing_when_out_of_aggro_range_of_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Target));
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(3., 0., 3.),
				Chase(*TARGET),
				Attack(*TARGET),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Entity(*TARGET),
				},
			))
			.id();

		_ = app.world_mut().run_system_once_with(
			select_behavior::<_Enemy, _Player, In<_GetRayCaster>>,
			_GetRayCaster::with_no_hit(),
		)?;

		let enemy = app.world().entity(enemy);
		assert_eq!((None, None), (enemy.get::<Chase>(), enemy.get::<Attack>()));
		Ok(())
	}

	#[test]
	fn keep_chasing_player_when_in_attack_range_but_no_los() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Player));
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.5),
				Chase(*PLAYER),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Player,
				},
			))
			.id();
		let get_ray_caster = _GetRayCaster::new().with_mock(|mock| {
			mock.expect_get_ray_caster().times(1).returning(|| {
				Ok(_RayCaster::new().with_mock(|mock| {
					mock.expect_cast_ray().return_const(None);
				}))
			});
		});

		_ = app.world_mut().run_system_once_with(
			select_behavior::<_Enemy, _Player, In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		let enemy = app.world().entity(enemy);
		assert_eq!(
			(Some(&Chase(*PLAYER)), None),
			(enemy.get::<Chase>(), enemy.get::<Attack>())
		);
		Ok(())
	}

	#[test]
	fn attack_player_when_in_attack_range_with_los() -> Result<(), RunSystemError> {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Player))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.5),
				Chase(*PLAYER),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Player,
				},
			))
			.id();
		let get_ray_caster = _GetRayCaster::new().with_mock(move |mock| {
			mock.expect_get_ray_caster().times(1).returning(move || {
				Ok(_RayCaster::new().with_mock(move |mock| {
					let arbitrary_toi = TimeOfImpact(42.);
					mock.expect_cast_ray()
						.return_const(Some((player, arbitrary_toi)));
				}))
			});
		});

		_ = app.world_mut().run_system_once_with(
			select_behavior::<_Enemy, _Player, In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		let enemy = app.world().entity(enemy);
		assert_eq!(
			(None, Some(&Attack(*PLAYER)),),
			(enemy.get::<Chase>(), enemy.get::<Attack>())
		);
		Ok(())
	}

	#[test]
	fn keep_chasing_player_when_in_attack_range_but_no_los_to_player_obstructed()
	-> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Player));
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.5),
				Chase(*PLAYER),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Player,
				},
			))
			.id();
		let get_ray_caster = _GetRayCaster::new().with_mock(|mock| {
			mock.expect_get_ray_caster().times(1).returning(|| {
				Ok(_RayCaster::new().with_mock(|mock| {
					let arbitrary_toi = TimeOfImpact(42.);
					let other_entity = Entity::from_raw(100);
					mock.expect_cast_ray()
						.return_const(Some((other_entity, arbitrary_toi)));
				}))
			});
		});

		_ = app.world_mut().run_system_once_with(
			select_behavior::<_Enemy, _Player, In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		let enemy = app.world().entity(enemy);
		assert_eq!(
			(Some(&Chase(*PLAYER)), None),
			(enemy.get::<Chase>(), enemy.get::<Attack>())
		);
		Ok(())
	}

	#[test]
	fn los_check_to_player_with_proper_ray() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Player));
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(0., 0., 1.),
				Chase(*PLAYER),
				_Enemy {
					attack_range: Units::new(10.).into(),
					aggro_range: Units::new(10.).into(),
					target: EnemyTarget::Player,
				},
			))
			.id();
		let get_ray_caster = _GetRayCaster::new().with_mock(move |mock| {
			mock.expect_get_ray_caster().times(1).returning(move || {
				Ok(_RayCaster::new().with_mock(move |mock| {
					let direction = Dir3::new(Vec3::new(1., 0., -1.)).expect("TEST DIR INVALID");
					mock.expect_cast_ray()
						.times(1)
						.with(eq((
							Ray3d {
								origin: Vec3::new(0., 0., 1.),
								direction,
							},
							NoSensors,
							ExcludeRigidBody(enemy),
						)))
						.return_const(None);
				}))
			});
		});

		_ = app.world_mut().run_system_once_with(
			select_behavior::<_Enemy, _Player, In<_GetRayCaster>>,
			get_ray_caster,
		)?;
		Ok(())
	}

	#[test]
	fn los_check_to_player_with_proper_ray_using_offsets() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((
			GlobalTransform::from_xyz(1., 0., 0.),
			_Player,
			GroundOffset(Vec3::new(0., 0.5, 0.)),
		));
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(0., 0., 1.),
				Chase(*PLAYER),
				_Enemy {
					attack_range: Units::new(10.).into(),
					aggro_range: Units::new(10.).into(),
					target: EnemyTarget::Player,
				},
				GroundOffset(Vec3::new(0., 1., 0.)),
			))
			.id();
		let get_ray_caster = _GetRayCaster::new().with_mock(move |mock| {
			mock.expect_get_ray_caster().times(1).returning(move || {
				Ok(_RayCaster::new().with_mock(move |mock| {
					let direction = Dir3::new(Vec3::new(1., -0.5, -1.)).expect("TEST DIR INVALID");
					mock.expect_cast_ray()
						.times(1)
						.with(eq((
							Ray3d {
								origin: Vec3::new(0., 1., 1.),
								direction,
							},
							NoSensors,
							ExcludeRigidBody(enemy),
						)))
						.return_const(None);
				}))
			});
		});

		_ = app.world_mut().run_system_once_with(
			select_behavior::<_Enemy, _Player, In<_GetRayCaster>>,
			get_ray_caster,
		)?;
		Ok(())
	}

	#[test]
	fn log_direction_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.spawn((GlobalTransform::from_xyz(f32::INFINITY, 0., 0.), _Player));
		app.world_mut().spawn((
			GlobalTransform::from_xyz(f32::INFINITY, 0., 0.5),
			Chase(*PLAYER),
			_Enemy {
				attack_range: Units::new(1.).into(),
				aggro_range: Units::new(2.).into(),
				target: EnemyTarget::Player,
			},
		));

		let result = app.world_mut().run_system_once_with(
			select_behavior::<_Enemy, _Player, In<_GetRayCaster>>,
			_GetRayCaster::with_no_hit(),
		)?;

		assert_eq!(
			Err(BehaviorError::InvalidDirectionErrors(vec![
				InvalidDirectionError::NaN
			])),
			result
		);
		Ok(())
	}

	#[test]
	fn attack_player_when_in_attack_range_with_los_via_collider_root() -> Result<(), RunSystemError>
	{
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Player))
			.id();
		let player_collider = app
			.world_mut()
			.spawn(ColliderOfInteractionTarget::from_raw(player))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.5),
				Chase(*PLAYER),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Player,
				},
			))
			.id();
		let get_ray_caster = _GetRayCaster::new().with_mock(move |mock| {
			mock.expect_get_ray_caster().times(1).returning(move || {
				Ok(_RayCaster::new().with_mock(move |mock| {
					let arbitrary_toi = TimeOfImpact(42.);
					mock.expect_cast_ray()
						.return_const(Some((player_collider, arbitrary_toi)));
				}))
			});
		});

		_ = app.world_mut().run_system_once_with(
			select_behavior::<_Enemy, _Player, In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		let enemy = app.world().entity(enemy);
		assert_eq!(
			(None, Some(&Attack(*PLAYER))),
			(enemy.get::<Chase>(), enemy.get::<Attack>())
		);
		Ok(())
	}
}
