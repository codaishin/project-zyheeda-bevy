use crate::{
	behaviors::{SkillBehaviorConfig, skill_shape::OnSkillStop},
	skills::lifetime_definition::LifeTimeDefinition,
	traits::spawn_skill::SpawnSkill,
};
use common::{
	components::lifetime::Lifetime,
	traits::handles_skill_physics::{Skill, SkillCaster, SkillSpawner, SkillTarget, Spawn},
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
		let mut skill = self.spawn(
			config.skill_contact(caster, spawner, target),
			config.skill_projection(),
		);

		for effect in config.contact.iter().map(|effect| (*effect).into()) {
			skill.insert_on_contact(effect);
		}

		for effect in config.projection.iter().map(|effect| (*effect).into()) {
			skill.insert_on_projection(effect);
		}

		match config.lifetime() {
			LifeTimeDefinition::Infinite => OnSkillStop::Ignore,
			LifeTimeDefinition::UntilStopped => OnSkillStop::Stop(skill.root()),
			LifeTimeDefinition::UntilOutlived(lifetime) => {
				skill.insert_on_root(Lifetime::from(lifetime));
				OnSkillStop::Ignore
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		behaviors::{
			attach_skill_effect::{AttachEffect, force::AttachForce},
			skill_shape::{SkillShape, ground_target::GroundTargetedAoe, shield::Shield},
		},
		skills::lifetime_definition::LifeTimeDefinition,
	};
	use bevy::ecs::bundle::Bundle;
	use common::{
		components::{lifetime::Lifetime, persistent_entity::PersistentEntity},
		effects::force::Force,
		tools::{Units, action_key::slot::SlotKey},
		traits::handles_skill_physics::{Contact, Effect, Projection, Skill},
	};
	use macros::simple_mock;
	use mockall::predicate::eq;
	use std::{sync::LazyLock, time::Duration};
	use test_case::test_case;
	use testing::Mock;

	simple_mock! {
		_Spawn {}
		impl Spawn for _Spawn {
			type TSkill<'c> = Mock_Skill where Self: 'c;
			fn spawn(&mut self, contact: Contact, projection: Projection) -> Mock_Skill;
		}
	}

	simple_mock! {
		_Skill {}
		impl Skill for _Skill {
			fn root(&self) -> PersistentEntity;
			fn insert_on_root<T>(&mut self, bundle: T) where T: Bundle;
			fn insert_on_contact(&mut self, effect: Effect);
			fn insert_on_projection(&mut self, effect: Effect);
		}
	}

	impl Mock_Skill {
		fn with_defaults(mut self) -> Self {
			self.expect_root().return_const(PersistentEntity::default());
			self.expect_insert_on_root::<Lifetime>().return_const(());
			self.expect_insert_on_contact().return_const(());
			self.expect_insert_on_projection().return_const(());

			self
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
				.with(
					eq(CONFIG.skill_contact(*CASTER, SPAWNER, *TARGET)),
					eq(CONFIG.skill_projection()),
				)
				.returning(|_, _| Mock_Skill::new().with_defaults());
		}
	}

	#[test]
	fn return_stoppable() {
		const CONFIG: SkillBehaviorConfig = ground_target(11, LifeTimeDefinition::UntilStopped);
		let root = PersistentEntity::default();
		let mut spawn = Mock_Spawn::new_mock(move |mock| {
			mock.expect_spawn().returning(move |_, _| {
				Mock_Skill::new_mock(|mock| {
					mock.expect_root().return_const(root);
				})
				.with_defaults()
			});
		});

		let on_skill_stop = spawn.spawn_skill(CONFIG, *CASTER, SPAWNER, *TARGET);

		assert_eq!(OnSkillStop::Stop(root), on_skill_stop);
	}

	#[test_case(ground_target(1, LifeTimeDefinition::Infinite); "infinite")]
	#[test_case(ground_target(1, LifeTimeDefinition::UntilOutlived(Duration::ZERO)); "with lifetime")]
	fn return_non_stoppable(config: SkillBehaviorConfig) {
		let root = PersistentEntity::default();
		let mut spawn = Mock_Spawn::new_mock(move |mock| {
			mock.expect_spawn().returning(move |_, _| {
				Mock_Skill::new_mock(|mock| {
					mock.expect_root().return_const(root);
				})
				.with_defaults()
			});
		});

		let on_skill_stop = spawn.spawn_skill(config, *CASTER, SPAWNER, *TARGET);

		assert_eq!(OnSkillStop::Ignore, on_skill_stop);
	}

	#[test]
	fn add_contact_effect() {
		let mut spawn = Mock_Spawn::new_mock(|mock| {
			mock.expect_spawn()
				.returning(|_, _| Mock_Skill::new_mock(assert_added_effects).with_defaults());
		});
		let config = SHIELD.with_contact_effects(vec![AttachEffect::Force(AttachForce)]);

		spawn.spawn_skill(config, *CASTER, SPAWNER, *TARGET);

		fn assert_added_effects(mock: &mut Mock_Skill) {
			mock.expect_insert_on_contact()
				.once()
				.with(eq(Effect::Force(Force)))
				.return_const(());
		}
	}

	#[test]
	fn add_projection_effect() {
		let mut spawn = Mock_Spawn::new_mock(|mock| {
			mock.expect_spawn()
				.returning(|_, _| Mock_Skill::new_mock(assert_added_effects).with_defaults());
		});
		let config = SHIELD.with_projection_effects(vec![AttachEffect::Force(AttachForce)]);

		spawn.spawn_skill(config, *CASTER, SPAWNER, *TARGET);

		fn assert_added_effects(mock: &mut Mock_Skill) {
			mock.expect_insert_on_projection()
				.once()
				.with(eq(Effect::Force(Force)))
				.return_const(());
		}
	}

	#[test]
	fn add_lifetime() {
		let mut spawn = Mock_Spawn::new_mock(|mock| {
			mock.expect_spawn()
				.returning(|_, _| Mock_Skill::new_mock(assert_added_effects).with_defaults());
		});
		let config = ground_target(1, LifeTimeDefinition::UntilOutlived(Duration::from_secs(2)));

		spawn.spawn_skill(config, *CASTER, SPAWNER, *TARGET);

		fn assert_added_effects(mock: &mut Mock_Skill) {
			mock.expect_insert_on_root()
				.once()
				.with(eq(Lifetime::from(Duration::from_secs(2))))
				.return_const(());
		}
	}
}
