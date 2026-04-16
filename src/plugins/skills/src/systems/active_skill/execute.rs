use crate::{
	components::active_skill::ActiveSkill,
	skills::shape::OnSkillStop,
	traits::spawn_skill::SpawnSkill,
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{
		handles_skill_physics::{Despawn, SkillCaster, SkillEntity},
		thread_safe::ThreadSafe,
	},
};

impl<TConfig> ActiveSkill<TConfig>
where
	TConfig: ThreadSafe + Clone,
{
	pub(crate) fn execute<TSpawn>(
		mut spawn: StaticSystemParam<TSpawn>,
		mut agents: Query<(Entity, &mut Self)>,
		persistent_entities: Query<&PersistentEntity>,
	) where
		TSpawn: for<'w, 's> SystemParam<Item<'w, 's>: SpawnSkill<TConfig> + Despawn>,
	{
		for (entity, mut skill_executer) in &mut agents {
			match skill_executer.as_ref() {
				Self::Start { slot_key, shape } => {
					let Ok(entity) = persistent_entities.get(entity) else {
						continue;
					};
					let on_stop_skill =
						spawn.spawn_skill(shape.clone(), SkillCaster(*entity), *slot_key);
					match on_stop_skill {
						OnSkillStop::Ignore => *skill_executer = Self::Idle,
						OnSkillStop::Stop(skill) => *skill_executer = Self::Stoppable(skill),
					}
				}
				Self::Stop(skill) => {
					spawn.despawn(SkillEntity(*skill));
					*skill_executer = Self::Idle
				}
				_ => {}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use common::{CommonPlugin, tools::action_key::slot::SlotKey};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::sync::LazyLock;
	use test_case::test_case;
	use testing::{IsChanged, NestedMocks, SingleThreadedApp};

	#[derive(Debug, PartialEq, Clone)]
	struct _Config;

	#[derive(Resource, NestedMocks)]
	struct _Spawner {
		mock: Mock_Spawner,
	}

	impl SpawnSkill<_Config> for ResMut<'_, _Spawner> {
		fn spawn_skill(
			&mut self,
			config: _Config,
			caster: SkillCaster,
			slot: SlotKey,
		) -> OnSkillStop {
			self.mock.spawn_skill(config, caster, slot)
		}
	}

	impl Despawn for _Spawner {
		fn despawn(&mut self, skill: SkillEntity) {
			self.mock.despawn(skill);
		}
	}

	mock! {
		_Spawner {}
		impl SpawnSkill<_Config> for _Spawner {
			fn spawn_skill(
				&mut self,
				config: _Config,
				caster: SkillCaster,
				slot: SlotKey,
			) -> OnSkillStop;
		}
		impl Despawn for _Spawner {
			fn despawn(&mut self, skill: SkillEntity) {}
		}
	}

	fn setup(spawner: _Spawner) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(spawner);
		app.add_plugins(CommonPlugin);
		app.add_systems(
			Update,
			(
				ActiveSkill::<_Config>::execute::<ResMut<_Spawner>>,
				IsChanged::<ActiveSkill<_Config>>::detect,
			)
				.chain(),
		);

		app
	}

	static CASTER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);
	static SKILL: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[test]
	fn spawn_started_skill() {
		let mut app = setup(_Spawner::new().with_mock(assert_call_spawn));
		app.world_mut().spawn((
			*CASTER,
			ActiveSkill::Start {
				slot_key: SlotKey(11),
				shape: _Config,
			},
		));

		app.update();

		fn assert_call_spawn(mock: &mut Mock_Spawner) {
			mock.expect_spawn_skill()
				.once()
				.with(eq(_Config), eq(SkillCaster(*CASTER)), eq(SlotKey(11)))
				.return_const(OnSkillStop::Ignore);
			mock.expect_despawn().return_const(());
		}
	}

	#[test_case(OnSkillStop::Ignore, ActiveSkill::Idle; "idle")]
	#[test_case(OnSkillStop::Stop(*SKILL), ActiveSkill::Stoppable(*SKILL); "stoppable")]
	fn set_started_to(on_skill_stop: OnSkillStop, expected: ActiveSkill<_Config>) {
		let mut app = setup(_Spawner::new().with_mock(|mock| {
			mock.expect_spawn_skill().return_const(on_skill_stop);
			mock.expect_despawn().return_const(());
		}));
		let entity = app
			.world_mut()
			.spawn((
				*CASTER,
				ActiveSkill::Start {
					slot_key: SlotKey(11),
					shape: _Config,
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&expected),
			app.world().entity(entity).get::<ActiveSkill<_Config>>(),
		);
	}

	#[test]
	fn despawn_stopped_skill() {
		let mut app = setup(_Spawner::new().with_mock(assert_call_despawn));
		app.world_mut()
			.spawn((*CASTER, ActiveSkill::<_Config>::Stop(*SKILL)));

		app.update();

		fn assert_call_despawn(mock: &mut Mock_Spawner) {
			mock.expect_spawn_skill().return_const(OnSkillStop::Ignore);
			mock.expect_despawn()
				.once()
				.with(eq(SkillEntity(*SKILL)))
				.return_const(());
		}
	}

	#[test]
	fn set_stopped_to_idle() {
		let mut app = setup(_Spawner::new().with_mock(|mock| {
			mock.expect_spawn_skill().return_const(OnSkillStop::Ignore);
			mock.expect_despawn().return_const(());
		}));
		let entity = app
			.world_mut()
			.spawn((*CASTER, ActiveSkill::<_Config>::Stop(*SKILL)))
			.id();

		app.update();

		assert_eq!(
			Some(&ActiveSkill::Idle),
			app.world().entity(entity).get::<ActiveSkill<_Config>>(),
		);
	}

	#[test_case(ActiveSkill::Idle; "idle")]
	#[test_case(ActiveSkill::Stoppable(*SKILL); "stoppable")]
	fn do_not_change(executor: ActiveSkill<_Config>) {
		let mut app = setup(_Spawner::new().with_mock(|mock| {
			mock.expect_spawn_skill().return_const(OnSkillStop::Ignore);
			mock.expect_despawn().return_const(());
		}));
		let entity = app.world_mut().spawn((*CASTER, executor)).id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(entity)
				.get::<IsChanged<ActiveSkill<_Config>>>(),
		);
	}
}
