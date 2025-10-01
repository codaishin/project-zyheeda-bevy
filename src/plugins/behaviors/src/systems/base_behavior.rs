use crate::components::{Attack, Chase};
use bevy::{math::InvalidDirectionError, prelude::*};
use bevy_rapier3d::plugin::ReadRapierContext;
use common::{
	components::{
		collider_relationship::ColliderOfInteractionTarget,
		ground_offset::GroundOffset,
		persistent_entity::PersistentEntity,
	},
	errors::{ErrorData, Level},
	tools::{aggro_range::AggroRange, attack_range::AttackRange},
	traits::{
		accessors::get::{DynProperty, GetMut, GetProperty},
		cast_ray::{
			CastRay,
			GetRayCaster,
			read_rapier_context::{ExcludeRigidBody, NoSensors},
		},
		handles_enemies::EnemyTarget,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Display;
use zyheeda_core::prelude::*;

impl<T> SelectBehavior for T {}

pub(crate) trait SelectBehavior {
	fn select_behavior<TPlayer>(
		rapier: ReadRapierContext,
		commands: ZyheedaCommands,
		agents: Query<(
			&PersistentEntity,
			&GlobalTransform,
			Option<&GroundOffset>,
			&Self,
		)>,
		players: Query<(&PersistentEntity, &GlobalTransform, Option<&GroundOffset>), With<TPlayer>>,
		all: Query<(&PersistentEntity, &GlobalTransform, Option<&GroundOffset>)>,
		colliders: Query<&ColliderOfInteractionTarget>,
	) -> Result<(), BehaviorError>
	where
		Self: Component
			+ Sized
			+ GetProperty<AggroRange>
			+ GetProperty<AttackRange>
			+ GetProperty<EnemyTarget>,
		TPlayer: Component,
	{
		select_behavior(rapier, commands, agents, players, all, colliders)
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum BehaviorError<TNoRayCaster = BevyError> {
	NoRayCaster(TNoRayCaster),
	InvalidDirectionErrors(Vec<InvalidDirectionError>),
}

impl Display for BehaviorError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			BehaviorError::NoRayCaster(error) => write!(f, "{error}"),
			BehaviorError::InvalidDirectionErrors(errors) => write_iter!(f, errors),
		}
	}
}

impl ErrorData for BehaviorError {
	type TDetails = Self;

	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> String {
		"Behavior error".to_owned()
	}

	fn into_details(self) -> Self::TDetails {
		self
	}
}

enum Behavior {
	Attack,
	Chase,
	Idle,
}

fn select_behavior<TAgent, TPlayer, TGetRayCaster>(
	ray_caster_source: TGetRayCaster,
	mut commands: ZyheedaCommands,
	agents: Query<(
		&PersistentEntity,
		&GlobalTransform,
		Option<&GroundOffset>,
		&TAgent,
	)>,
	players: Query<(&PersistentEntity, &GlobalTransform, Option<&GroundOffset>), With<TPlayer>>,
	all: Query<(&PersistentEntity, &GlobalTransform, Option<&GroundOffset>)>,
	colliders: Query<&ColliderOfInteractionTarget>,
) -> Result<(), BehaviorError<TGetRayCaster::TError>>
where
	TAgent: Component
		+ Sized
		+ GetProperty<AggroRange>
		+ GetProperty<AttackRange>
		+ GetProperty<EnemyTarget>,
	TPlayer: Component,
	TGetRayCaster: GetRayCaster<(Ray3d, NoSensors, ExcludeRigidBody)>,
{
	let ray_caster = match ray_caster_source.get_ray_caster() {
		Ok(ray_caster) => ray_caster,
		Err(error) => return Err(BehaviorError::NoRayCaster(error)),
	};
	let mut invalid_directions = vec![];

	for (persistent_agent, transform, ground_offset, agent) in &agents {
		let target = match agent.dyn_property::<EnemyTarget>() {
			EnemyTarget::Player => players.single().ok(),
			EnemyTarget::Entity(persistent_entity) => commands
				.get_mut(&persistent_entity)
				.and_then(|entity| all.get(entity.id()).ok()),
		};
		let Some((persistent_target, target_transform, target_ground_offset)) = target else {
			continue;
		};
		let Some(target_entity) = commands.get_mut(persistent_target).map(|e| e.id()) else {
			continue;
		};
		let Some(agent_entity) = commands.get_mut(persistent_agent).map(|e| e.id()) else {
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
		let Some(mut agent_entity) = commands.get_mut(&agent_entity) else {
			continue;
		};

		match strategy {
			Err(invalid_direction) => invalid_directions.push(invalid_direction),
			Ok(Behavior::Attack) => {
				agent_entity.try_insert(Attack(*persistent_target));
				agent_entity.try_remove::<Chase>();
			}
			Ok(Behavior::Chase) => {
				agent_entity.try_insert(Chase(*persistent_target));
				agent_entity.try_remove::<Attack>();
			}
			Ok(Behavior::Idle) => {
				agent_entity.try_remove::<Chase>();
				agent_entity.try_remove::<Attack>();
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
	TAgent: GetProperty<AggroRange> + GetProperty<AttackRange>,
	TCaster: CastRay<(Ray3d, NoSensors, ExcludeRigidBody)>,
{
	let direction = target_translation - enemy_translation;
	let distance = direction.length();

	if distance > *enemy_agent.dyn_property::<AggroRange>() {
		return Ok(Behavior::Idle);
	}
	if distance > *enemy_agent.dyn_property::<AttackRange>() {
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

	impl GetProperty<AggroRange> for _Enemy {
		fn get_property(&self) -> Units {
			self.aggro_range.0
		}
	}

	impl GetProperty<AttackRange> for _Enemy {
		fn get_property(&self) -> Units {
			self.attack_range.0
		}
	}

	impl GetProperty<EnemyTarget> for _Enemy {
		fn get_property(&self) -> EnemyTarget {
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
					attack_range: Units::from(1.).into(),
					aggro_range: Units::from(2.).into(),
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
					attack_range: Units::from(1.).into(),
					aggro_range: Units::from(2.).into(),
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
					attack_range: Units::from(1.).into(),
					aggro_range: Units::from(2.).into(),
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
					attack_range: Units::from(1.).into(),
					aggro_range: Units::from(2.).into(),
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
					attack_range: Units::from(1.).into(),
					aggro_range: Units::from(2.).into(),
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
					attack_range: Units::from(1.).into(),
					aggro_range: Units::from(2.).into(),
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
					attack_range: Units::from(1.).into(),
					aggro_range: Units::from(2.).into(),
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
					attack_range: Units::from(10.).into(),
					aggro_range: Units::from(10.).into(),
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
					attack_range: Units::from(10.).into(),
					aggro_range: Units::from(10.).into(),
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
				attack_range: Units::from(1.).into(),
				aggro_range: Units::from(2.).into(),
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
					attack_range: Units::from(1.).into(),
					aggro_range: Units::from(2.).into(),
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
