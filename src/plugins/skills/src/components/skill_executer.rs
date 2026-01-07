mod dto;

use crate::{
	behaviors::spawn_skill::{OnSkillStop, SpawnOn},
	skills::{RunSkillBehavior, dto::run_skill_behavior::RunSkillBehaviorDto},
	traits::{Execute, Flush, Schedule, spawn_loadout_skill::SpawnLoadoutSkill},
};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::TryApplyOn,
		handles_physics::HandlesAllPhysicalEffects,
		handles_skill_physics::{HandlesNewPhysicalSkill, SkillCaster, SkillSpawner, SkillTarget},
	},
	zyheeda_commands::ZyheedaCommands,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[savable_component(dto = SkillExecuter<RunSkillBehaviorDto>)]
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

impl<TSpawnSkill, TPhysics> Execute<TPhysics> for SkillExecuter<TSpawnSkill>
where
	TSpawnSkill: SpawnLoadoutSkill,
	TPhysics: HandlesAllPhysicalEffects + HandlesNewPhysicalSkill + 'static,
{
	fn execute(
		&mut self,
		commands: &mut ZyheedaCommands,
		caster: SkillCaster,
		target: SkillTarget,
	) {
		match self {
			SkillExecuter::Start { shape, slot_key } => {
				let spawner = match shape.spawn_on() {
					SpawnOn::Center => SkillSpawner::Neutral,
					SpawnOn::Slot => SkillSpawner::Slot(*slot_key),
				};
				let on_skill_stop_behavior =
					shape.spawn::<TPhysics>(commands, caster, spawner, target);

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

fn stop<TSkillShape>(
	skill: PersistentEntity,
	commands: &mut ZyheedaCommands,
) -> SkillExecuter<TSkillShape> {
	commands.try_apply_on(&skill, |e| e.try_despawn());
	SkillExecuter::Idle
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce, SystemParam};
	use common::{
		attributes::health::Health,
		components::persistent_entity::PersistentEntity,
		tools::action_key::slot::{PlayerSlot, Side},
		traits::{
			accessors::get::{GetContextMut, GetProperty},
			handles_physics::{Effect as EffectTrait, HandlesPhysicalEffect},
			handles_skill_physics::{
				Contact,
				Effect,
				HandlesNewPhysicalSkill,
				NewSkill,
				Projection,
				Skill,
				SkillEntities,
				SkillRoot,
				Spawn,
			},
			register_persistent_entities::RegisterPersistentEntities,
			thread_safe::ThreadSafe,
		},
	};
	use std::sync::{Arc, Mutex};
	use testing::{SingleThreadedApp, assert_count};

	struct _Commands;

	struct _HandlesPhysics;

	impl<T> HandlesPhysicalEffect<T> for _HandlesPhysics
	where
		T: EffectTrait + ThreadSafe,
	{
		type TEffectComponent = _Effect;
		type TAffectedComponent = _Affected;

		fn into_effect_component(_: T) -> _Effect {
			_Effect
		}
	}

	impl HandlesNewPhysicalSkill for _HandlesPhysics {
		type TSkillSpawnerMut<'w, 's> = _SkillSpawner;

		fn spawn_skill(commands: &mut ZyheedaCommands, _: Contact, _: Projection) -> SkillEntities {
			SkillEntities {
				root: SkillRoot {
					entity: commands.spawn(()).id(),
					persistent_entity: PersistentEntity::default(),
				},
				contact: commands.spawn(_Contact).id(),
				projection: commands.spawn(_Projection).id(),
			}
		}
	}

	#[derive(Component)]
	struct _Effect;

	#[derive(Component)]
	struct _Affected;

	impl GetProperty<Health> for _Affected {
		fn get_property(&self) -> Health {
			panic!("NOT USED")
		}
	}

	#[derive(Component)]
	struct _Contact;

	#[derive(Component)]
	struct _Projection;

	#[derive(SystemParam)]
	struct _SkillSpawner;

	impl GetContextMut<NewSkill> for _SkillSpawner {
		type TContext<'ctx> = _Context;

		fn get_context_mut<'ctx>(
			_: &'ctx mut _SkillSpawner,
			_: NewSkill,
		) -> Option<Self::TContext<'ctx>> {
			None
		}
	}

	struct _Context;

	impl Spawn for _Context {
		type TSkill<'c>
			= _SpawnedSkill
		where
			Self: 'c;

		fn spawn(&mut self, _: Contact, _: Projection) -> Self::TSkill<'_> {
			panic!("SHOULD NOT BE CALLED")
		}
	}

	struct _SpawnedSkill;

	impl Skill for _SpawnedSkill {
		fn root(&self) -> PersistentEntity {
			panic!("SHOULD NOT BE CALLED")
		}

		fn insert_on_root<T>(&mut self, _: T)
		where
			T: Bundle,
		{
			panic!("SHOULD NOT BE CALLED")
		}

		fn insert_on_contact(&mut self, _: Effect) {
			panic!("SHOULD NOT BE CALLED")
		}

		fn insert_on_projection(&mut self, _: Effect) {
			panic!("SHOULD NOT BE CALLED")
		}
	}

	#[derive(Debug, PartialEq, Clone)]
	struct _ShapeSlotted(OnSkillStop);

	impl SpawnLoadoutSkill for _ShapeSlotted {
		fn spawn_on(&self) -> SpawnOn {
			SpawnOn::Slot
		}

		fn spawn<TPhysics>(
			&self,
			_: &mut ZyheedaCommands,
			_: SkillCaster,
			_: SkillSpawner,
			_: SkillTarget,
		) -> OnSkillStop
		where
			TPhysics: HandlesAllPhysicalEffects + HandlesNewPhysicalSkill + 'static,
		{
			self.0
		}
	}

	type _Executer<'a> = &'a mut dyn Execute<_HandlesPhysics>;

	#[derive(Debug, PartialEq)]
	struct _Behavior {
		spawn_on: SpawnOn,
		on_skill_stop: OnSkillStop,
	}

	impl SpawnLoadoutSkill for _Behavior {
		fn spawn_on(&self) -> SpawnOn {
			self.spawn_on
		}

		fn spawn<TEffects>(
			&self,
			commands: &mut ZyheedaCommands,
			caster: SkillCaster,
			spawner: SkillSpawner,
			target: SkillTarget,
		) -> OnSkillStop
		where
			TEffects: HandlesAllPhysicalEffects + HandlesNewPhysicalSkill + 'static,
		{
			commands.spawn(_SpawnedBehavior {
				caster,
				spawner,
				target,
			});
			self.on_skill_stop
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _SpawnedBehavior {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	}

	fn as_executer(e: &mut SkillExecuter<_Behavior>) -> &mut impl Execute<_HandlesPhysics> {
		e
	}

	fn spawned_behavior(app: &App) -> impl Iterator<Item = &'_ _SpawnedBehavior> + '_ {
		app.world()
			.iter_entities()
			.filter_map(|e| e.get::<_SpawnedBehavior>())
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.register_persistent_entities();
		app
	}

	#[test]
	fn set_self_to_start_skill() {
		let shape = _ShapeSlotted(OnSkillStop::Ignore);
		let slot_key = SlotKey::from(PlayerSlot::Lower(Side::Left));

		let mut executer = SkillExecuter::default();
		executer.schedule(slot_key, shape.clone());

		assert_eq!(SkillExecuter::Start { slot_key, shape }, executer);
	}

	#[test]
	fn start_shape_on_slot() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Slot(SlotKey::from(PlayerSlot::Lower(Side::Right)));
		let target = SkillTarget::Entity(PersistentEntity::default());
		let mut executer = SkillExecuter::Start {
			slot_key: SlotKey::from(PlayerSlot::Lower(Side::Right)),
			shape: _Behavior {
				spawn_on: SpawnOn::Slot,
				on_skill_stop: OnSkillStop::Ignore,
			},
		};

		app.world_mut()
			.run_system_once(move |mut commands: ZyheedaCommands| {
				as_executer(&mut executer).execute(&mut commands, caster, target);
			})?;

		let [behavior] = assert_count!(1, spawned_behavior(&app));
		assert_eq!(
			&_SpawnedBehavior {
				caster,
				spawner,
				target
			},
			behavior,
		);
		Ok(())
	}

	#[test]
	fn start_shape_on_center() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget::Entity(PersistentEntity::default());
		let mut executer: SkillExecuter<_Behavior> = SkillExecuter::Start {
			slot_key: SlotKey::from(PlayerSlot::Lower(Side::Right)),
			shape: _Behavior {
				spawn_on: SpawnOn::Center,
				on_skill_stop: OnSkillStop::Ignore,
			},
		};

		app.world_mut()
			.run_system_once(move |mut commands: ZyheedaCommands| {
				as_executer(&mut executer).execute(&mut commands, caster, target);
			})?;

		let [behavior] = assert_count!(1, spawned_behavior(&app));
		assert_eq!(
			&_SpawnedBehavior {
				caster,
				spawner,
				target
			},
			behavior,
		);
		Ok(())
	}

	#[test]
	fn set_to_idle_when_ignore_on_skill_stop() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let target = SkillTarget::Entity(PersistentEntity::default());
		let executer = Arc::new(Mutex::new(SkillExecuter::Start {
			slot_key: SlotKey::from(PlayerSlot::Lower(Side::Right)),
			shape: _Behavior {
				spawn_on: SpawnOn::Center,
				on_skill_stop: OnSkillStop::Ignore,
			},
		}));
		let mutex = executer.clone();

		app.world_mut()
			.run_system_once(move |mut commands: ZyheedaCommands| {
				let Ok(mut lock) = mutex.lock() else {
					return;
				};
				as_executer(&mut lock).execute(&mut commands, caster, target);
			})?;

		assert_eq!(SkillExecuter::Idle, *executer.lock().unwrap());
		Ok(())
	}

	#[test]
	fn set_to_stoppable_when_stop_on_skill_stop() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let target = SkillTarget::Entity(PersistentEntity::default());
		let entity = PersistentEntity::default();
		let executer = Arc::new(Mutex::new(SkillExecuter::Start {
			slot_key: SlotKey::from(PlayerSlot::Lower(Side::Right)),
			shape: _Behavior {
				spawn_on: SpawnOn::Center,
				on_skill_stop: OnSkillStop::Stop(entity),
			},
		}));
		let mutex = executer.clone();

		app.world_mut()
			.run_system_once(move |mut commands: ZyheedaCommands| {
				let Ok(mut lock) = mutex.lock() else {
					return;
				};
				as_executer(&mut lock).execute(&mut commands, caster, target);
			})?;

		assert_eq!(
			SkillExecuter::StartedStoppable(entity),
			*executer.lock().unwrap()
		);
		Ok(())
	}

	#[test]
	fn set_to_idle_on_flush_when_set_to_start() {
		let mut executer = SkillExecuter::Start {
			slot_key: SlotKey::from(PlayerSlot::Lower(Side::Right)),
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
	fn despawn_skill_entity_recursively_on_execute_stop() -> Result<(), RunSystemError> {
		let mut app = setup();
		let skill = PersistentEntity::default();
		let caster = SkillCaster(PersistentEntity::default());
		let target = SkillTarget::Entity(PersistentEntity::default());
		let mut executer = SkillExecuter::<_Behavior>::Stop(skill);
		let entity = app.world_mut().spawn(skill).id();

		app.world_mut()
			.run_system_once(move |mut commands: ZyheedaCommands| {
				as_executer(&mut executer).execute(&mut commands, caster, target);
			})?;

		assert!(app.world().get_entity(entity).is_err());
		Ok(())
	}

	#[test]
	fn set_to_idle_on_stop_execution() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let target = SkillTarget::Entity(PersistentEntity::default());
		let entity = PersistentEntity::default();
		let executer = Arc::new(Mutex::new(SkillExecuter::Stop(entity)));
		let mutex = executer.clone();

		app.world_mut()
			.run_system_once(move |mut commands: ZyheedaCommands| {
				let Ok(mut lock) = mutex.lock() else {
					return;
				};
				as_executer(&mut lock).execute(&mut commands, caster, target);
			})?;

		assert_eq!(SkillExecuter::Idle, *executer.lock().unwrap());
		Ok(())
	}
}
