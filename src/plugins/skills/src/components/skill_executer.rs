pub(crate) mod dto;

use super::SkillTarget;
use crate::{
	behaviors::{SkillCaster, build_skill_shape::OnSkillStop, spawn_on::SpawnOn},
	skills::RunSkillBehavior,
	traits::{Execute, Flush, Schedule, spawn_skill_behavior::SpawnSkillBehavior},
};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	tools::action_key::slot::SlotKey,
	traits::{
		handles_effect::HandlesAllEffects,
		handles_lifetime::HandlesLifetime,
		handles_skill_behaviors::{HandlesSkillBehaviors, Spawner},
		try_despawn::TryDespawnPersistent,
	},
};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) enum SkillExecuter<TSkillBehavior = RunSkillBehavior> {
	#[default]
	Idle,
	Start {
		slot_key: SlotKey,
		shape: TSkillBehavior,
	},
	StartedStoppable(PersistentEntity),
	Stop(PersistentEntity),
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

impl<TCommands, TBehavior, TLifetimes, TEffects, TSkillBehavior>
	Execute<TCommands, TLifetimes, TEffects, TSkillBehavior> for SkillExecuter<TBehavior>
where
	TBehavior: SpawnSkillBehavior<TCommands>,
	TCommands: TryDespawnPersistent,
	TLifetimes: HandlesLifetime + 'static,
	TEffects: HandlesAllEffects + 'static,
	TSkillBehavior: HandlesSkillBehaviors + 'static,
{
	fn execute(&mut self, commands: &mut TCommands, caster: &SkillCaster, target: &SkillTarget) {
		match self {
			SkillExecuter::Start { shape, slot_key } => {
				let spawner = match shape.spawn_on() {
					SpawnOn::Center => Spawner::Center,
					SpawnOn::Slot => Spawner::Slot(*slot_key),
				};
				let on_skill_stop_behavior = shape.spawn::<TLifetimes, TEffects, TSkillBehavior>(
					commands, caster, spawner, target,
				);

				*self = match on_skill_stop_behavior {
					OnSkillStop::Ignore => SkillExecuter::Idle,
					OnSkillStop::Stop(entity) => SkillExecuter::StartedStoppable(entity),
				};
			}
			SkillExecuter::Stop(skill) => {
				*self = stop(*skill, commands);
			}
			_ => {}
		};
	}
}

fn stop<TCommands, TSkillShape>(
	skill: PersistentEntity,
	commands: &mut TCommands,
) -> SkillExecuter<TSkillShape>
where
	TCommands: TryDespawnPersistent,
{
	commands.try_despawn_persistent(skill);
	SkillExecuter::Idle
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::behaviors::spawn_on::SpawnOn;
	use common::{
		components::{outdated::Outdated, persistent_entity::PersistentEntity},
		simple_init,
		tools::{action_key::slot::Side, collider_info::ColliderInfo},
		traits::{
			handles_effect::HandlesEffect,
			handles_skill_behaviors::{Contact, Projection, SkillEntities, SkillRoot},
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
		type TTarget = ();
		type TEffectComponent = _Effect;

		fn effect(_: T) -> _Effect {
			_Effect
		}

		fn attribute(_: Self::TTarget) -> impl Bundle {}
	}

	#[derive(Component)]
	struct _Effect;

	struct _HandlesSkillBehaviors;

	impl HandlesSkillBehaviors for _HandlesSkillBehaviors {
		type TSkillContact = _Contact;
		type TSkillProjection = _Projection;

		fn spawn_skill(commands: &mut Commands, _: Contact, _: Projection) -> SkillEntities {
			SkillEntities {
				root: SkillRoot {
					entity: commands.spawn_empty().id(),
					persistent_entity: PersistentEntity::default(),
				},
				contact: commands.spawn(_Contact).id(),
				projection: commands.spawn(_Projection).id(),
			}
		}
	}

	#[derive(Component)]
	struct _Contact;

	#[derive(Component)]
	struct _Projection;

	impl TryDespawnPersistent for _Commands {
		fn try_despawn_persistent(&mut self, _: PersistentEntity) {}
	}

	mock! {
		_Commands {}
		impl TryDespawnPersistent for _Commands {
			fn try_despawn_persistent(&mut self, entity: PersistentEntity);
		}
	}

	simple_init!(Mock_Commands);

	#[derive(Debug, PartialEq, Clone)]
	struct _ShapeSlotted(OnSkillStop);

	impl SpawnSkillBehavior<_Commands> for _ShapeSlotted {
		fn spawn_on(&self) -> SpawnOn {
			SpawnOn::Slot
		}

		fn spawn<TLifetimes, TEffects, TSkillBehaviors>(
			&self,
			_: &mut _Commands,
			_: &SkillCaster,
			_: Spawner,
			_: &SkillTarget,
		) -> OnSkillStop
		where
			TLifetimes: HandlesLifetime + 'static,
			TEffects: HandlesAllEffects + 'static,
			TSkillBehaviors: HandlesSkillBehaviors + 'static,
		{
			self.0.clone()
		}
	}

	mock! {
		_Behavior {}
		impl SpawnSkillBehavior<Mock_Commands> for _Behavior {
			fn spawn_on(&self) -> SpawnOn;
			fn spawn<TLifetimes, TEffects, TSkillBehaviors>(
				&self,
				commands: &mut Mock_Commands,
				caster: &SkillCaster,
				spawner: Spawner,
				target: &SkillTarget,
			) -> OnSkillStop
			where
				TLifetimes: HandlesLifetime + 'static,
				TEffects: HandlesAllEffects + 'static,
				TSkillBehaviors: HandlesSkillBehaviors + 'static;
		}
	}

	simple_init!(Mock_Behavior);

	type _Executer<'a, TCommands> =
		&'a mut dyn Execute<TCommands, _HandlesLifetimes, _HandlesEffects, _HandlesSkillBehaviors>;

	fn get_target() -> SkillTarget {
		SkillTarget {
			ray: Ray3d::new(
				Vec3::new(1., 2., 3.),
				Dir3::new_unchecked(Vec3::new(4., 5., 6.).normalize()),
			),
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
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = Spawner::Slot(SlotKey::BottomHand(Side::Right));
		let target = get_target();

		let executer: _Executer<Mock_Commands> = &mut SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: Mock_Behavior::new_mock(|mock| {
				mock.expect_spawn_on().return_const(SpawnOn::Slot);
				mock.expect_spawn::<_HandlesLifetimes, _HandlesEffects, _HandlesSkillBehaviors>()
					.withf(move |_, c, s, t| {
						assert_eq!((&caster, &spawner, &target), (c, s, t));
						true
					})
					.return_const(OnSkillStop::Ignore);
			}),
		};

		executer.execute(&mut Mock_Commands::new(), &caster, &target);
	}

	#[test]
	fn start_shape_on_center() {
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = Spawner::Center;
		let target = get_target();
		let mut commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_despawn_persistent().return_const(());
		});

		let executer: _Executer<Mock_Commands> = &mut SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: Mock_Behavior::new_mock(|mock| {
				mock.expect_spawn_on().return_const(SpawnOn::Center);
				mock.expect_spawn::<_HandlesLifetimes, _HandlesEffects, _HandlesSkillBehaviors>()
					.withf(move |_, c, s, t| {
						assert_eq!((&caster, &spawner, &target), (c, s, t));
						true
					})
					.return_const(OnSkillStop::Ignore);
			}),
		};

		executer.execute(&mut commands, &caster, &target);
	}

	#[test]
	fn set_to_idle_when_ignore_on_skill_stop() {
		let caster = SkillCaster(PersistentEntity::default());
		let target = get_target();

		let mut executer = SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: _ShapeSlotted(OnSkillStop::Ignore),
		};

		{
			let executer: _Executer<_Commands> = &mut executer;
			executer.execute(&mut _Commands, &caster, &target);
		}

		assert_eq!(SkillExecuter::Idle, executer);
	}

	#[test]
	fn set_to_stoppable_when_stop_on_skill_stop() {
		let caster = SkillCaster(PersistentEntity::default());
		let target = get_target();
		let skill = PersistentEntity::default();

		let mut executer = SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: _ShapeSlotted(OnSkillStop::Stop(skill)),
		};

		{
			let executer: _Executer<_Commands> = &mut executer;
			executer.execute(&mut _Commands, &caster, &target);
		}

		assert_eq!(SkillExecuter::StartedStoppable(skill), executer);
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
		let skill = PersistentEntity::default();
		let mut executer = SkillExecuter::<_ShapeSlotted>::StartedStoppable(skill);

		executer.flush();

		assert_eq!(SkillExecuter::Stop(skill), executer);
	}

	#[test]
	fn despawn_skill_entity_recursively_on_execute_stop() {
		let skill = PersistentEntity::default();
		let caster = SkillCaster(PersistentEntity::default());
		let target = get_target();
		let executer: _Executer<Mock_Commands> = &mut SkillExecuter::<Mock_Behavior>::Stop(skill);

		let mut commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_despawn_persistent()
				.times(1)
				.with(eq(skill))
				.return_const(());
		});

		executer.execute(&mut commands, &caster, &target);
	}

	#[test]
	fn set_to_idle_on_stop_execution() {
		let skill = PersistentEntity::default();
		let caster = SkillCaster(PersistentEntity::default());
		let target = get_target();
		let mut commands = _Commands;
		let mut executer = SkillExecuter::<_ShapeSlotted>::Stop(skill);

		{
			let executer: _Executer<_Commands> = &mut executer;
			executer.execute(&mut commands, &caster, &target);
		}

		assert_eq!(SkillExecuter::Idle, executer);
	}
}
