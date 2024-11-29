use super::skill_spawners::SkillSpawners;
use crate::{
	behaviors::{
		build_skill_shape::OnSkillStop,
		spawn_on::SpawnOn,
		SkillCaster,
		SkillSpawner,
		Target,
	},
	skills::RunSkillBehavior,
	slot_key::SlotKey,
	traits::{spawn_skill_behavior::SpawnSkillBehavior, Execute, Flush, Schedule},
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	traits::{
		accessors::get::GetRef,
		handles_effect::HandlesAllEffects,
		handles_effect_shading::HandlesEffectShadingForAll,
		handles_lifetime::HandlesLifetime,
		try_despawn_recursive::TryDespawnRecursive,
	},
};

#[derive(Component, Debug, PartialEq, Default, Clone)]
pub(crate) enum SkillExecuter<TSkillBehavior = RunSkillBehavior> {
	#[default]
	Idle,
	Start {
		slot_key: SlotKey,
		shape: TSkillBehavior,
	},
	StartedStoppable(Entity),
	Stop(Entity),
}

impl<TBehavior> Schedule<TBehavior> for SkillExecuter<TBehavior> {
	fn schedule(&mut self, slot_key: SlotKey, behavior: TBehavior) {
		*self = SkillExecuter::Start {
			slot_key,
			shape: behavior,
		};
	}
}

impl<TBehavior> Flush for SkillExecuter<TBehavior> {
	fn flush(&mut self) {
		match self {
			SkillExecuter::StartedStoppable(entity) => {
				*self = SkillExecuter::Stop(*entity);
			}
			SkillExecuter::Start { .. } => {
				*self = SkillExecuter::Idle;
			}
			_ => {}
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct NoSkillSpawner(Option<SlotKey>);

impl From<NoSkillSpawner> for Error {
	fn from(NoSkillSpawner(slot): NoSkillSpawner) -> Self {
		Error {
			msg: format!("Could not find spawner for Slot: {slot:?}"),
			lvl: Level::Error,
		}
	}
}

impl<TCommands, TBehavior, TLifetimeDependency, TEffectDependency, TShaderDependency>
	Execute<TCommands, TLifetimeDependency, TEffectDependency, TShaderDependency>
	for SkillExecuter<TBehavior>
where
	TBehavior: SpawnSkillBehavior<TCommands>,
	TCommands: TryDespawnRecursive,
	TLifetimeDependency: HandlesLifetime + 'static,
	TEffectDependency: HandlesAllEffects + 'static,
	TShaderDependency: HandlesEffectShadingForAll + 'static,
{
	type TError = NoSkillSpawner;

	fn execute(
		&mut self,
		commands: &mut TCommands,
		caster: &SkillCaster,
		spawners: &SkillSpawners,
		target: &Target,
	) -> Result<(), Self::TError> {
		match self {
			SkillExecuter::Start { shape, slot_key } => {
				let spawner = get_spawner(shape, spawners, *slot_key)?;
				let on_skill_stop_behavior = shape
					.spawn::<TLifetimeDependency, TEffectDependency, TShaderDependency>(
						commands, caster, spawner, target,
					);

				*self = match on_skill_stop_behavior {
					OnSkillStop::Ignore => SkillExecuter::Idle,
					OnSkillStop::Stop(entity) => SkillExecuter::StartedStoppable(entity),
				};
			}
			SkillExecuter::Stop(skills) => {
				*self = stop(skills, commands);
			}
			_ => {}
		};

		Ok(())
	}
}

fn get_spawner<'a, TCommands, TSkillShape>(
	shape: &TSkillShape,
	spawners: &'a SkillSpawners,
	slot_key: SlotKey,
) -> Result<&'a SkillSpawner, NoSkillSpawner>
where
	TSkillShape: SpawnSkillBehavior<TCommands>,
{
	let slot_key = match shape.spawn_on() {
		SpawnOn::Center => None,
		SpawnOn::Slot => Some(slot_key),
	};
	match spawners.get(&slot_key) {
		Some(spawner) => Ok(spawner),
		None => Err(NoSkillSpawner(slot_key)),
	}
}

fn stop<TCommands, TSkillShape>(
	skill: &Entity,
	commands: &mut TCommands,
) -> SkillExecuter<TSkillShape>
where
	TCommands: TryDespawnRecursive,
{
	commands.try_despawn_recursive(*skill);
	SkillExecuter::Idle
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{behaviors::spawn_on::SpawnOn, components::skill_spawners::SkillSpawners};
	use common::{
		components::{Outdated, Side},
		resources::ColliderInfo,
		simple_init,
		traits::{
			handles_effect::HandlesEffect,
			handles_effect_shading::HandlesEffectShadingFor,
			mock::Mock,
		},
	};
	use mockall::{mock, predicate::eq};
	use std::time::Duration;

	struct _Commands;

	struct _HandlesLifetimes;

	impl HandlesLifetime for _HandlesLifetimes {
		fn lifetime(_: Duration) -> impl Bundle {}
	}

	struct _HandlesEffects;

	impl<T> HandlesEffect<T> for _HandlesEffects
	where
		T: Sync + Send + 'static,
	{
		fn effect(_: T) -> impl Bundle {}
	}

	struct _HandlesShading;

	impl<T> HandlesEffectShadingFor<T> for _HandlesShading
	where
		T: Sync + Send + 'static,
	{
		fn effect_shader(_: T) -> impl Bundle {}
	}

	impl TryDespawnRecursive for _Commands {
		fn try_despawn_recursive(&mut self, _: Entity) {}
	}

	mock! {
		_Commands {}
		impl TryDespawnRecursive for _Commands {
			fn try_despawn_recursive(&mut self, entity: Entity);
		}
	}

	simple_init!(Mock_Commands);

	#[derive(Debug, PartialEq, Clone)]
	struct _ShapeSlotted(OnSkillStop);

	impl SpawnSkillBehavior<_Commands> for _ShapeSlotted {
		fn spawn_on(&self) -> SpawnOn {
			SpawnOn::Slot
		}

		fn spawn<TLifetimeDependency, TEffectDependency, TShaderDependency>(
			&self,
			_: &mut _Commands,
			_: &SkillCaster,
			_: &SkillSpawner,
			_: &Target,
		) -> OnSkillStop
		where
			TLifetimeDependency: HandlesLifetime + 'static,
			TEffectDependency: HandlesAllEffects + 'static,
			TShaderDependency: HandlesEffectShadingForAll + 'static,
		{
			self.0.clone()
		}
	}

	mock! {
		_Behavior {}
		impl SpawnSkillBehavior<Mock_Commands> for _Behavior {
			fn spawn_on(&self) -> SpawnOn;
			fn spawn<TLifetimeDependency, TEffectDependency, TShaderDependency>(
				&self,
				commands: &mut Mock_Commands,
				caster: &SkillCaster,
				spawner: &SkillSpawner,
				target: &Target,
			) -> OnSkillStop
			where
				TLifetimeDependency: HandlesLifetime + 'static,
				TEffectDependency: HandlesAllEffects + 'static,
			TShaderDependency: HandlesEffectShadingForAll + 'static;
		}
	}

	simple_init!(Mock_Behavior);

	type _Executer<'a, TCommands> = &'a mut dyn Execute<
		TCommands,
		_HandlesLifetimes,
		_HandlesEffects,
		_HandlesShading,
		TError = NoSkillSpawner,
	>;

	fn get_target() -> Target {
		Target {
			ray: Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(4., 5., 6.)),
			collision_info: Some(ColliderInfo {
				collider: Outdated {
					entity: Entity::from_raw(11),
					component: GlobalTransform::from_xyz(10., 10., 10.),
				},
				root: Some(Outdated {
					entity: Entity::from_raw(1),
					component: GlobalTransform::from_xyz(11., 11., 11.),
				}),
			}),
		}
	}

	#[test]
	fn set_self_to_start_skill() {
		let shape = _ShapeSlotted(OnSkillStop::Ignore);
		let slot_key = SlotKey::BottomHand(Side::Left);

		let mut executer = SkillExecuter::default();
		executer.schedule(slot_key, shape.clone());

		assert_eq!(SkillExecuter::Start { slot_key, shape }, executer);
	}

	#[test]
	fn start_shape_on_slot() {
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(Some(SlotKey::BottomHand(Side::Right)), spawner)]);
		let target = get_target();

		let executer: _Executer<Mock_Commands> = &mut SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: Mock_Behavior::new_mock(|mock| {
				mock.expect_spawn_on().return_const(SpawnOn::Slot);
				mock.expect_spawn::<_HandlesLifetimes, _HandlesEffects, _HandlesShading>()
					.withf(move |_, c, s, t| {
						assert_eq!((&caster, &spawner, &target), (c, s, t));
						true
					})
					.return_const(OnSkillStop::Ignore);
			}),
		};

		_ = executer.execute(&mut Mock_Commands::new(), &caster, &spawners, &target);
	}

	#[test]
	fn start_shape_on_center() {
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();
		let mut commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_despawn_recursive().return_const(());
		});

		let executer: _Executer<Mock_Commands> = &mut SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: Mock_Behavior::new_mock(|mock| {
				mock.expect_spawn_on().return_const(SpawnOn::Center);
				mock.expect_spawn::<_HandlesLifetimes, _HandlesEffects, _HandlesShading>()
					.withf(move |_, c, s, t| {
						assert_eq!((&caster, &spawner, &target), (c, s, t));
						true
					})
					.return_const(OnSkillStop::Ignore);
			}),
		};

		_ = executer.execute(&mut commands, &caster, &spawners, &target);
	}

	#[test]
	fn set_to_idle_when_ignore_on_skill_stop() {
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(Some(SlotKey::BottomHand(Side::Right)), spawner)]);
		let target = get_target();

		let mut executer = SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: _ShapeSlotted(OnSkillStop::Ignore),
		};

		{
			let executer: _Executer<_Commands> = &mut executer;
			_ = executer.execute(&mut _Commands, &caster, &spawners, &target);
		}

		assert_eq!(SkillExecuter::Idle, executer);
	}

	#[test]
	fn set_to_stoppable_when_stop_on_skill_stop() {
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(Some(SlotKey::BottomHand(Side::Right)), spawner)]);
		let target = get_target();

		let mut executer = SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: _ShapeSlotted(OnSkillStop::Stop(Entity::from_raw(123))),
		};

		{
			let executer: _Executer<_Commands> = &mut executer;
			_ = executer.execute(&mut _Commands, &caster, &spawners, &target);
		}

		assert_eq!(
			SkillExecuter::StartedStoppable(Entity::from_raw(123)),
			executer
		);
	}

	#[test]
	fn slot_lookup_error_on_start() {
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();
		let executer: _Executer<Mock_Commands> = &mut SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: Mock_Behavior::new_mock(|mock| {
				mock.expect_spawn_on().return_const(SpawnOn::Slot);
				mock.expect_spawn::<_HandlesLifetimes, _HandlesEffects, _HandlesShading>()
					.return_const(OnSkillStop::Ignore);
			}),
		};

		let result = executer.execute(&mut Mock_Commands::new(), &caster, &spawners, &target);

		assert_eq!(
			Err(NoSkillSpawner(Some(SlotKey::BottomHand(Side::Right)))),
			result
		);
	}

	#[test]
	fn set_to_idle_on_flush_when_set_to_start() {
		let mut executer = SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: _ShapeSlotted(OnSkillStop::Ignore),
		};

		executer.flush();

		assert_eq!(SkillExecuter::Idle, executer);
	}

	#[test]
	fn set_to_stop_on_flush_when_set_to_started() {
		let mut executer = SkillExecuter::<_ShapeSlotted>::StartedStoppable(Entity::from_raw(1));

		executer.flush();

		assert_eq!(SkillExecuter::Stop(Entity::from_raw(1)), executer);
	}

	#[test]
	fn despawn_skill_entity_recursively_on_execute_stop() {
		let caster = SkillCaster(Entity::from_raw(1));
		let spawners = SkillSpawners::new([]);
		let target = get_target();
		let executer: _Executer<Mock_Commands> =
			&mut SkillExecuter::<Mock_Behavior>::Stop(Entity::from_raw(123));

		let mut commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_despawn_recursive()
				.times(1)
				.with(eq(Entity::from_raw(123)))
				.return_const(());
		});

		_ = executer.execute(&mut commands, &caster, &spawners, &target);
	}

	#[test]
	fn set_to_idle_on_stop_execution() {
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();
		let mut commands = _Commands;
		let mut executer = SkillExecuter::<_ShapeSlotted>::Stop(Entity::from_raw(1));

		{
			let executer: _Executer<_Commands> = &mut executer;
			_ = executer.execute(&mut commands, &caster, &spawners, &target);
		}

		assert_eq!(SkillExecuter::Idle, executer);
	}
}
