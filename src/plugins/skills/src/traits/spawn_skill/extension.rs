use crate::{
	skills::{
		behaviors::SkillBehaviorConfig,
		lifetime_definition::LifeTimeDefinition,
		shape::OnSkillStop,
	},
	traits::spawn_skill::SpawnSkill,
};
use common::traits::handles_skill_physics::{
	SkillCaster,
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
		mut config: SkillBehaviorConfig,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> OnSkillStop {
		let (stoppable, lifetime) = match config.lifetime() {
			LifeTimeDefinition::Infinite => (false, None),
			LifeTimeDefinition::UntilStopped => (true, None),
			LifeTimeDefinition::UntilOutlived(lifetime) => (false, Some(lifetime)),
		};

		let entity = self.spawn(SpawnArgs {
			contact: config.skill_contact(caster, spawner, target),
			projection: config.skill_projection(),
			lifetime,
			contact_effects: config.take_skill_contact_effects(),
			projection_effects: config.take_skill_projection_effects(),
		});

		match stoppable {
			true => OnSkillStop::Stop(entity),
			false => OnSkillStop::Ignore,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::skills::{
		lifetime_definition::LifeTimeDefinition,
		shape::{SkillShape, ground_target::GroundTargetedAoe, shield::Shield},
	};
	use common::{
		components::persistent_entity::PersistentEntity,
		effects::force::Force,
		tools::{Units, action_key::slot::SlotKey},
		traits::handles_skill_physics::Effect,
	};
	use macros::simple_mock;
	use mockall::predicate::eq;
	use std::{sync::LazyLock, time::Duration};
	use test_case::test_case;
	use testing::Mock;

	simple_mock! {
		_Spawn {}
		impl Spawn for _Spawn {
			fn spawn(&mut self, args: SpawnArgs) -> PersistentEntity;
		}
	}

	const fn ground_target(radius: u8, lifetime: LifeTimeDefinition) -> SkillBehaviorConfig {
		SkillBehaviorConfig::from_shape(SkillShape::GroundTargetedAoe(GroundTargetedAoe {
			max_range: Units::from_u8(u8::MAX),
			radius: Units::from_u8(radius),
			lifetime,
		}))
	}

	const SHIELD: SkillBehaviorConfig = SkillBehaviorConfig::from_shape(SkillShape::Shield(Shield));

	static CASTER: LazyLock<SkillCaster> =
		LazyLock::new(|| SkillCaster(PersistentEntity::default()));
	const SPAWNER: SkillSpawner = SkillSpawner::Slot(SlotKey(123));
	static TARGET: LazyLock<SkillTarget> =
		LazyLock::new(|| SkillTarget::Entity(PersistentEntity::default()));

	#[test]
	fn spawn_contact_and_projection() {
		const CONFIG: SkillBehaviorConfig = ground_target(11, LifeTimeDefinition::Infinite);
		let mut spawn = Mock_Spawn::new_mock(assert_contact_and_projection_used);

		spawn.spawn_skill(CONFIG, *CASTER, SPAWNER, *TARGET);

		fn assert_contact_and_projection_used(mock: &mut Mock_Spawn) {
			mock.expect_spawn()
				.once()
				.with(eq(SpawnArgs::with_shape(
					CONFIG.skill_contact(*CASTER, SPAWNER, *TARGET),
					CONFIG.skill_projection(),
				)))
				.return_const(PersistentEntity::default());
		}
	}

	#[test]
	fn return_stoppable() {
		const CONFIG: SkillBehaviorConfig = ground_target(11, LifeTimeDefinition::UntilStopped);
		let entity = PersistentEntity::default();
		let mut spawn = Mock_Spawn::new_mock(move |mock| {
			mock.expect_spawn().return_const(entity);
		});

		let on_skill_stop = spawn.spawn_skill(CONFIG, *CASTER, SPAWNER, *TARGET);

		assert_eq!(OnSkillStop::Stop(entity), on_skill_stop);
	}

	#[test_case(ground_target(1, LifeTimeDefinition::Infinite); "infinite")]
	#[test_case(ground_target(1, LifeTimeDefinition::UntilOutlived(Duration::ZERO)); "with lifetime")]
	fn return_non_stoppable(config: SkillBehaviorConfig) {
		let mut spawn = Mock_Spawn::new_mock(move |mock| {
			mock.expect_spawn()
				.return_const(PersistentEntity::default());
		});

		let on_skill_stop = spawn.spawn_skill(config, *CASTER, SPAWNER, *TARGET);

		assert_eq!(OnSkillStop::Ignore, on_skill_stop);
	}

	#[test]
	fn add_contact_effect() {
		let config = SHIELD.with_contact_effects(vec![Effect::Force(Force)]);
		let mut spawn = Mock_Spawn::new_mock(assert_added_effects(config.clone()));

		spawn.spawn_skill(config, *CASTER, SPAWNER, *TARGET);

		fn assert_added_effects(mut config: SkillBehaviorConfig) -> impl FnMut(&mut Mock_Spawn) {
			move |mock| {
				mock.expect_spawn()
					.once()
					.with(eq(SpawnArgs::with_shape(
						config.skill_contact(*CASTER, SPAWNER, *TARGET),
						config.skill_projection(),
					)
					.with_contact_effects(config.take_skill_contact_effects())))
					.return_const(PersistentEntity::default());
			}
		}
	}

	#[test]
	fn add_projection_effect() {
		let config = SHIELD.with_projection_effects(vec![Effect::Force(Force)]);
		let mut spawn = Mock_Spawn::new_mock(assert_added_effects(config.clone()));

		spawn.spawn_skill(config, *CASTER, SPAWNER, *TARGET);

		fn assert_added_effects(mut config: SkillBehaviorConfig) -> impl FnMut(&mut Mock_Spawn) {
			move |mock| {
				mock.expect_spawn()
					.once()
					.with(eq(SpawnArgs::with_shape(
						config.skill_contact(*CASTER, SPAWNER, *TARGET),
						config.skill_projection(),
					)
					.with_projection_effects(config.take_skill_projection_effects())))
					.return_const(PersistentEntity::default());
			}
		}
	}

	#[test]
	fn add_lifetime() {
		let config = ground_target(1, LifeTimeDefinition::UntilOutlived(Duration::from_secs(2)));
		let mut spawn = Mock_Spawn::new_mock(assert_lifetime(config.clone()));

		spawn.spawn_skill(config, *CASTER, SPAWNER, *TARGET);

		fn assert_lifetime(config: SkillBehaviorConfig) -> impl Fn(&mut Mock_Spawn) {
			move |mock| {
				mock.expect_spawn()
					.once()
					.with(eq(SpawnArgs::with_shape(
						config.skill_contact(*CASTER, SPAWNER, *TARGET),
						config.skill_projection(),
					)
					.with_lifetime(Duration::from_secs(2))))
					.return_const(PersistentEntity::default());
			}
		}
	}
}
