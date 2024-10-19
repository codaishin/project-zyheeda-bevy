use crate::{
	behaviors::{
		build_skill_shape::OnSkillStop,
		spawn_on::SpawnOn,
		SkillCaster,
		SkillSpawner,
		Target,
	},
	items::slot_key::SlotKey,
	skills::RunSkillBehavior,
	traits::{spawn_skill_behavior::SpawnSkillBehavior, Execute, Flush, Schedule},
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	traits::{get::GetRef, try_despawn_recursive::TryDespawnRecursive},
};

use super::skill_spawners::SkillSpawners;

#[derive(Component, Debug, PartialEq, Default, Clone)]
pub(crate) enum SkillExecuter<TSkillBehavior = RunSkillBehavior> {
	#[default]
	Idle,
	Start {
		slot_key: SlotKey,
		behavior: TSkillBehavior,
	},
	StartedStoppable(Entity),
	Stop(Entity),
}

impl<TBehavior> Schedule<TBehavior> for SkillExecuter<TBehavior> {
	fn schedule(&mut self, slot_key: SlotKey, behavior: TBehavior) {
		*self = SkillExecuter::Start { slot_key, behavior };
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

impl<TCommands, TBehavior> Execute<TCommands> for SkillExecuter<TBehavior>
where
	TBehavior: SpawnSkillBehavior<TCommands>,
	TCommands: TryDespawnRecursive,
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
			SkillExecuter::Start {
				behavior: shape,
				slot_key,
			} => {
				let spawner = get_spawner(shape, spawners, *slot_key)?;
				*self = match shape.spawn(commands, caster, spawner, target) {
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
		traits::mock::Mock,
	};
	use mockall::{mock, predicate::eq};

	struct _Commands;

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
	struct _BehaviorSlotted(OnSkillStop);

	impl SpawnSkillBehavior<_Commands> for _BehaviorSlotted {
		fn spawn_on(&self) -> SpawnOn {
			SpawnOn::Slot
		}

		fn spawn(
			&self,
			_: &mut _Commands,
			_: &SkillCaster,
			_: &SkillSpawner,
			_: &Target,
		) -> OnSkillStop {
			self.0.clone()
		}
	}

	mock! {
		_Behavior {}
		impl SpawnSkillBehavior<Mock_Commands> for _Behavior {
			fn spawn_on(&self) -> SpawnOn;
			fn spawn(
				&self,
				commands: &mut Mock_Commands,
				caster: &SkillCaster,
				spawner: &SkillSpawner,
				target: &Target,
			) -> OnSkillStop;
		}
	}

	simple_init!(Mock_Behavior);

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
		let behavior = _BehaviorSlotted(OnSkillStop::Ignore);
		let slot_key = SlotKey::BottomHand(Side::Left);

		let mut executer = SkillExecuter::default();
		executer.schedule(slot_key, behavior.clone());

		assert_eq!(SkillExecuter::Start { slot_key, behavior }, executer);
	}

	#[test]
	fn start_shape_on_slot() {
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(Some(SlotKey::BottomHand(Side::Right)), spawner)]);
		let target = get_target();

		let mut executer = SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			behavior: Mock_Behavior::new_mock(|mock| {
				mock.expect_spawn_on().return_const(SpawnOn::Slot);
				mock.expect_spawn()
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

		let mut executer = SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			behavior: Mock_Behavior::new_mock(|mock| {
				mock.expect_spawn_on().return_const(SpawnOn::Center);
				mock.expect_spawn()
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
			behavior: _BehaviorSlotted(OnSkillStop::Ignore),
		};

		_ = executer.execute(&mut _Commands, &caster, &spawners, &target);

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
			behavior: _BehaviorSlotted(OnSkillStop::Stop(Entity::from_raw(123))),
		};

		_ = executer.execute(&mut _Commands, &caster, &spawners, &target);

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
		let mut executer = SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			behavior: Mock_Behavior::new_mock(|mock| {
				mock.expect_spawn_on().return_const(SpawnOn::Slot);
				mock.expect_spawn().return_const(OnSkillStop::Ignore);
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
			behavior: _BehaviorSlotted(OnSkillStop::Ignore),
		};

		executer.flush();

		assert_eq!(SkillExecuter::Idle, executer);
	}

	#[test]
	fn set_to_stop_on_flush_when_set_to_started() {
		let mut executer = SkillExecuter::<_BehaviorSlotted>::StartedStoppable(Entity::from_raw(1));

		executer.flush();

		assert_eq!(SkillExecuter::Stop(Entity::from_raw(1)), executer);
	}

	#[test]
	fn despawn_skill_entity_recursively_on_execute_stop() {
		let caster = SkillCaster(Entity::from_raw(1));
		let spawners = SkillSpawners::new([]);
		let target = get_target();
		let mut executer = SkillExecuter::<Mock_Behavior>::Stop(Entity::from_raw(123));

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
		let mut executer = SkillExecuter::<_BehaviorSlotted>::Stop(Entity::from_raw(1));

		_ = executer.execute(&mut commands, &caster, &spawners, &target);

		assert_eq!(SkillExecuter::Idle, executer);
	}
}
