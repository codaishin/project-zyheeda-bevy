use crate::{
	skills::{behaviors::SkillBehaviorConfig, shape::OnSkillStop},
	traits::spawn_skill::SpawnSkill,
};
use common::traits::handles_skill_physics::{
	SkillCaster,
	SkillShape,
	SkillSpawner,
	SkillTarget,
	Spawn,
	SpawnArgs,
};

impl<T> SpawnSkill<SkillBehaviorConfig> for T
where
	T: Spawn,
{
	fn spawn_skill(
		&mut self,
		config: SkillBehaviorConfig,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> OnSkillStop {
		let entity = self.spawn(SpawnArgs {
			shape: &config.shape,
			contact_effects: &config.contact,
			projection_effects: &config.projection,
			caster,
			spawner,
			target,
		});

		match config.shape {
			SkillShape::Shield(..) | SkillShape::Beam(..) => OnSkillStop::Stop(entity),
			_ => OnSkillStop::Ignore,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		components::persistent_entity::PersistentEntity,
		effects::force::Force,
		tools::{Units, action_key::slot::SlotKey},
		traits::{
			handles_physics::physical_bodies::Blockers,
			handles_skill_physics::{
				Effect,
				SkillShape,
				beam::Beam,
				ground_target::SphereAoE,
				shield::Shield,
			},
		},
	};
	use macros::simple_mock;
	use std::time::Duration;
	use test_case::test_case;
	use testing::Mock;
	use uuid::uuid;

	simple_mock! {
		_Spawn {}
		impl Spawn for _Spawn {
			fn spawn<'a>(&'a mut self, args: SpawnArgs<'a>) -> PersistentEntity;
		}
	}

	const fn ground_target(radius: u8, lifetime: Option<Duration>) -> SkillBehaviorConfig {
		SkillBehaviorConfig::from_shape(SkillShape::SphereAoE(SphereAoE {
			max_range: Units::from_u8(u8::MAX),
			radius: Units::from_u8(radius),
			lifetime,
		}))
	}

	const SHIELD: SkillBehaviorConfig = SkillBehaviorConfig::from_shape(SkillShape::Shield(Shield));

	static CASTER: SkillCaster = SkillCaster(PersistentEntity::from_uuid(uuid!(
		"91409ebe-e94d-43b2-8dc1-53949e4f0dc2"
	)));
	const SPAWNER: SkillSpawner = SkillSpawner::Slot(SlotKey(123));
	static TARGET: SkillTarget = SkillTarget::Entity(PersistentEntity::from_uuid(uuid!(
		"d0032d1c-a840-4ea5-9047-86f88a0437dc"
	)));

	#[test]
	fn spawn_contact_and_projection() {
		const CONFIG: SkillBehaviorConfig = ground_target(11, None);
		const ARGS: SpawnArgs = SpawnArgs {
			shape: &CONFIG.shape,
			caster: CASTER,
			spawner: SPAWNER,
			target: TARGET,
			contact_effects: &[],
			projection_effects: &[],
		};
		let mut spawn = Mock_Spawn::new_mock(assert_contact_and_projection_used);

		spawn.spawn_skill(CONFIG, CASTER, SPAWNER, TARGET);

		fn assert_contact_and_projection_used(mock: &mut Mock_Spawn) {
			mock.expect_spawn()
				.once()
				.withf(|args| args == &ARGS)
				.return_const(PersistentEntity::default());
		}
	}

	#[test_case(SkillShape::Shield(Shield); "shield")]
	#[test_case(SkillShape::Beam(Beam {range: Units::ZERO, blocked_by: Blockers::All}); "beam")]
	fn return_stoppable(shape: SkillShape) {
		let config = SkillBehaviorConfig::from_shape(shape);
		let entity = PersistentEntity::default();
		let mut spawn = Mock_Spawn::new_mock(move |mock| {
			mock.expect_spawn().return_const(entity);
		});

		let on_skill_stop = spawn.spawn_skill(config, CASTER, SPAWNER, TARGET);

		assert_eq!(OnSkillStop::Stop(entity), on_skill_stop);
	}

	#[test_case(ground_target(1, None); "infinite")]
	#[test_case(ground_target(1, Some(Duration::ZERO)); "with lifetime")]
	fn return_non_stoppable(config: SkillBehaviorConfig) {
		let mut spawn = Mock_Spawn::new_mock(move |mock| {
			mock.expect_spawn()
				.return_const(PersistentEntity::default());
		});

		let on_skill_stop = spawn.spawn_skill(config, CASTER, SPAWNER, TARGET);

		assert_eq!(OnSkillStop::Ignore, on_skill_stop);
	}

	#[test]
	fn add_contact_effect() {
		let config = SHIELD.with_contact_effects(vec![Effect::Force(Force)]);
		let mut spawn = Mock_Spawn::new_mock(assert_added_effects);

		spawn.spawn_skill(config, CASTER, SPAWNER, TARGET);

		fn assert_added_effects(mock: &mut Mock_Spawn) {
			mock.expect_spawn()
				.once()
				.withf(|args| args.contact_effects == [Effect::Force(Force)])
				.return_const(PersistentEntity::default());
		}
	}

	#[test]
	fn add_projection_effect() {
		let config = SHIELD.with_projection_effects(vec![Effect::Force(Force)]);
		let mut spawn = Mock_Spawn::new_mock(assert_added_effects);

		spawn.spawn_skill(config, CASTER, SPAWNER, TARGET);

		fn assert_added_effects(mock: &mut Mock_Spawn) {
			mock.expect_spawn()
				.once()
				.withf(|args| args.projection_effects == [Effect::Force(Force)])
				.return_const(PersistentEntity::default());
		}
	}
}
