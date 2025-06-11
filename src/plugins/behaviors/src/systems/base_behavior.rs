use crate::components::{Attack, Chase};
use bevy::{math::InvalidDirectionError, prelude::*};
use bevy_rapier3d::plugin::ReadRapierContext;
use common::{
	components::{
		collider_relationship::ColliderOfInteractionTarget,
		ground_offset::GroundOffset,
		persistent_entity::PersistentEntity,
	},
	resources::persistent_entities::PersistentEntities,
	tools::{
		aggro_range::AggroRange,
		attack_range::AttackRange,
		exclude_rigid_body::ExcludeRigidBody,
	},
	traits::{
		accessors::get::Getter,
		cast_ray::{CastRay, GetRayCaster},
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
	) -> BehaviorResults
	where
		Self: Component + Sized + Getter<AggroRange> + Getter<AttackRange> + Getter<EnemyTarget>,
		TPlayer: Component,
	{
		BehaviorResults(select_behavior(
			rapier,
			commands,
			agents,
			players,
			all,
			colliders,
			persistent_entities,
		))
	}
}

pub(crate) struct BehaviorResults(Result<Vec<Result<(), InvalidDirectionError>>, BevyError>);

impl TryFrom<BehaviorResults> for Vec<Result<(), InvalidDirectionError>> {
	type Error = BevyError;

	fn try_from(BehaviorResults(results): BehaviorResults) -> Result<Self, Self::Error> {
		results
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
) -> Result<Vec<Result<(), InvalidDirectionError>>, TGetRayCaster::TError>
where
	TAgent: Component + Sized + Getter<AggroRange> + Getter<AttackRange> + Getter<EnemyTarget>,
	TPlayer: Component,
	TGetRayCaster: GetRayCaster<(Ray3d, ExcludeRigidBody)>,
{
	let player = players.single().ok();
	let ray_caster = ray_caster_source.get_ray_caster()?;
	let mut results = vec![];

	for (persistent_agent_entity, transform, ground_offset, agent) in &agents {
		let target = match agent.get() {
			EnemyTarget::Player => player,
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
			Err(error) => results.push(Err(error)),
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

	Ok(results)
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
	TCaster: CastRay<(Ray3d, ExcludeRigidBody)>,
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

	match ray_caster.cast_ray(&(ray, ExcludeRigidBody(enemy))) {
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
		test_tools::utils::SingleThreadedApp,
		tools::Units,
		traits::{
			cast_ray::TimeOfImpact,
			clamp_zero_positive::ClampZeroPositive,
			nested_mock::NestedMocks,
			register_persistent_entities::RegisterPersistentEntities,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::sync::LazyLock;

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

	pub enum _ContextQueryError {}

	#[automock]
	impl GetRayCaster<(Ray3d, ExcludeRigidBody)> for _GetRayCaster {
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
	impl CastRay<(Ray3d, ExcludeRigidBody)> for _RayCaster {
		fn cast_ray(&self, ray_data: &(Ray3d, ExcludeRigidBody)) -> Option<(Entity, TimeOfImpact)> {
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

		Ok(_) = app.world_mut().run_system_once_with(
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

		Ok(_) = app.world_mut().run_system_once_with(
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

		Ok(_) = app.world_mut().run_system_once_with(
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

		Ok(_) = app.world_mut().run_system_once_with(
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

		Ok(_) = app.world_mut().run_system_once_with(
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

		Ok(_) = app.world_mut().run_system_once_with(
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

		Ok(_) = app.world_mut().run_system_once_with(
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
							ExcludeRigidBody(enemy),
						)))
						.return_const(None);
				}))
			});
		});

		Ok(_) = app.world_mut().run_system_once_with(
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
							ExcludeRigidBody(enemy),
						)))
						.return_const(None);
				}))
			});
		});

		Ok(_) = app.world_mut().run_system_once_with(
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

		let Ok(errors) = app.world_mut().run_system_once_with(
			select_behavior::<_Enemy, _Player, In<_GetRayCaster>>,
			_GetRayCaster::with_no_hit(),
		)?;

		assert_eq!(vec![Err(InvalidDirectionError::NaN)], errors);
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

		Ok(_) = app.world_mut().run_system_once_with(
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
