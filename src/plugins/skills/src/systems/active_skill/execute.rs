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
		handles_physics::{MouseHover, MouseHoversOver, Raycast},
		handles_skill_physics::{Despawn, SkillCaster, SkillEntity, SkillSpawner, SkillTarget},
		thread_safe::ThreadSafe,
	},
};

impl<TConfig> ActiveSkill<TConfig>
where
	TConfig: ThreadSafe + Clone,
{
	pub(crate) fn execute<TSpawn, TRaycast>(
		mut spawn: StaticSystemParam<TSpawn>,
		mut ray_cast: StaticSystemParam<TRaycast>,
		mut agents: Query<(Entity, &mut Self)>,
		persistent_entities: Query<&PersistentEntity>,
	) where
		TSpawn: for<'w, 's> SystemParam<Item<'w, 's>: SpawnSkill<TConfig> + Despawn>,
		TRaycast: for<'w, 's> SystemParam<Item<'w, 's>: Raycast<MouseHover>>,
	{
		for (entity, mut skill_executer) in &mut agents {
			match skill_executer.as_ref() {
				Self::Start { slot_key, shape } => {
					let Some(target) = get_target(&mut ray_cast, &persistent_entities) else {
						continue;
					};
					let Ok(entity) = persistent_entities.get(entity) else {
						continue;
					};
					let on_stop_skill = spawn.spawn_skill(
						shape.clone(),
						SkillCaster(*entity),
						SkillSpawner::Slot(*slot_key),
						target,
					);
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

fn get_target<TRaycast>(
	ray_cast: &mut StaticSystemParam<TRaycast>,
	persistent_entities: &Query<&PersistentEntity>,
) -> Option<SkillTarget>
where
	TRaycast: for<'w, 's> SystemParam<Item<'w, 's>: Raycast<MouseHover>>,
{
	match ray_cast.raycast(MouseHover::NO_EXCLUDES)? {
		MouseHoversOver::Ground { point } => Some(SkillTarget::from(point)),
		MouseHoversOver::Object { entity, .. } => persistent_entities
			.get(entity)
			.ok()
			.map(|e| SkillTarget::from(*e)),
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use common::{CommonPlugin, tools::action_key::slot::SlotKey};
	use macros::NestedMocks;
	use mockall::{automock, mock, predicate::eq};
	use std::sync::LazyLock;
	use test_case::test_case;
	use testing::{IsChanged, NestedMocks, SingleThreadedApp};

	#[derive(Debug, PartialEq, Clone)]
	struct _Config;

	#[derive(Resource, NestedMocks)]
	struct _RayCaster {
		mock: Mock_RayCaster,
	}

	impl Default for _RayCaster {
		fn default() -> Self {
			let mut mock = Mock_RayCaster::new();
			mock.expect_raycast()
				.return_const(MouseHoversOver::Ground { point: Vec3::ZERO });

			Self { mock }
		}
	}

	#[automock]
	impl Raycast<MouseHover> for _RayCaster {
		fn raycast(&mut self, args: MouseHover) -> Option<MouseHoversOver> {
			self.mock.raycast(args)
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Spawner {
		mock: Mock_Spawner,
	}

	impl SpawnSkill<_Config> for ResMut<'_, _Spawner> {
		fn spawn_skill(
			&mut self,
			config: _Config,
			caster: SkillCaster,
			spawner: SkillSpawner,
			target: SkillTarget,
		) -> OnSkillStop {
			self.mock.spawn_skill(config, caster, spawner, target)
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
				spawner: SkillSpawner,
				target: SkillTarget,
			) -> OnSkillStop;
		}
		impl Despawn for _Spawner {
			fn despawn(&mut self, skill: SkillEntity) {}
		}
	}

	fn setup(ray_caster: _RayCaster, spawner: _Spawner) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(ray_caster);
		app.insert_resource(spawner);
		app.add_plugins(CommonPlugin);
		app.add_systems(
			Update,
			(
				ActiveSkill::<_Config>::execute::<ResMut<_Spawner>, ResMut<_RayCaster>>,
				IsChanged::<ActiveSkill<_Config>>::detect,
			)
				.chain(),
		);

		app
	}

	static CASTER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);
	static SKILL: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);
	static TARGET: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[test]
	fn spawn_started_skill_on_ground() {
		let mut app = setup(
			_RayCaster::new().with_mock(|mock| {
				mock.expect_raycast().return_const(MouseHoversOver::Ground {
					point: Vec3::new(1., 2., 3.),
				});
			}),
			_Spawner::new().with_mock(assert_call_spawn),
		);
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
				.with(
					eq(_Config),
					eq(SkillCaster(*CASTER)),
					eq(SkillSpawner::Slot(SlotKey(11))),
					eq(SkillTarget::Ground(Vec3::new(1., 2., 3.))),
				)
				.return_const(OnSkillStop::Ignore);
			mock.expect_despawn().return_const(());
		}
	}

	#[test]
	fn spawn_started_skill_on_object() {
		let mut app = setup(
			_RayCaster::default(),
			_Spawner::new().with_mock(assert_call_spawn),
		);
		let target = app.world_mut().spawn(*TARGET).id();
		app.insert_resource(_RayCaster::new().with_mock(|mock| {
			mock.expect_raycast().return_const(MouseHoversOver::Object {
				entity: target,
				point: Vec3::ZERO,
			});
		}));
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
				.with(
					eq(_Config),
					eq(SkillCaster(*CASTER)),
					eq(SkillSpawner::Slot(SlotKey(11))),
					eq(SkillTarget::Entity(*TARGET)),
				)
				.return_const(OnSkillStop::Ignore);
			mock.expect_despawn().return_const(());
		}
	}

	#[test_case(OnSkillStop::Ignore, ActiveSkill::Idle; "idle")]
	#[test_case(OnSkillStop::Stop(*SKILL), ActiveSkill::Stoppable(*SKILL); "stoppable")]
	fn set_started_to(on_skill_stop: OnSkillStop, expected: ActiveSkill<_Config>) {
		let mut app = setup(
			_RayCaster::new().with_mock(|mock| {
				mock.expect_raycast().return_const(MouseHoversOver::Ground {
					point: Vec3::new(1., 2., 3.),
				});
			}),
			_Spawner::new().with_mock(|mock| {
				mock.expect_spawn_skill().return_const(on_skill_stop);
				mock.expect_despawn().return_const(());
			}),
		);
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
		let mut app = setup(
			_RayCaster::default(),
			_Spawner::new().with_mock(assert_call_despawn),
		);
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
		let mut app = setup(
			_RayCaster::new().with_mock(|mock| {
				mock.expect_raycast().return_const(MouseHoversOver::Ground {
					point: Vec3::new(1., 2., 3.),
				});
			}),
			_Spawner::new().with_mock(|mock| {
				mock.expect_spawn_skill().return_const(OnSkillStop::Ignore);
				mock.expect_despawn().return_const(());
			}),
		);
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

	#[test]
	fn call_raycast_with_no_excludes() {
		let mut app = setup(
			_RayCaster::new().with_mock(assert_call_with_no_excludes),
			_Spawner::new().with_mock(|mock| {
				mock.expect_spawn_skill().return_const(OnSkillStop::Ignore);
				mock.expect_despawn().return_const(());
			}),
		);
		app.world_mut().spawn((
			*CASTER,
			ActiveSkill::Start {
				slot_key: SlotKey(11),
				shape: _Config,
			},
		));

		app.update();

		fn assert_call_with_no_excludes(mock: &mut Mock_RayCaster) {
			mock.expect_raycast()
				.once()
				.with(eq(MouseHover::NO_EXCLUDES))
				.return_const(MouseHoversOver::Ground { point: Vec3::ZERO });
		}
	}

	#[test_case(ActiveSkill::Idle; "idle")]
	#[test_case(ActiveSkill::Stoppable(*SKILL); "stoppable")]
	fn do_not_change(executor: ActiveSkill<_Config>) {
		let mut app = setup(
			_RayCaster::new().with_mock(|mock| {
				mock.expect_raycast().return_const(MouseHoversOver::Ground {
					point: Vec3::new(1., 2., 3.),
				});
			}),
			_Spawner::new().with_mock(|mock| {
				mock.expect_spawn_skill().return_const(OnSkillStop::Ignore);
				mock.expect_despawn().return_const(());
			}),
		);
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
