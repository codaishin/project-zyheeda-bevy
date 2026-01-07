use crate::{
	behaviors::skill_shape::OnSkillStop,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::spawn_skill::{SkillData, SpawnSkill},
};
use common::{
	components::lifetime::Lifetime,
	traits::handles_skill_physics::{Skill, SkillCaster, SkillSpawner, SkillTarget, Spawn},
};

impl<T, TSkillConfig> SpawnSkill<TSkillConfig> for T
where
	T: Spawn,
	TSkillConfig: SkillData,
{
	fn spawn_skill(
		&mut self,
		config: TSkillConfig,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> OnSkillStop {
		let mut skill = self.spawn(
			config.skill_contact(caster, spawner, target),
			config.skill_projection(),
		);

		for effect in config.skill_contact_effects() {
			skill.insert_on_contact(effect);
		}

		for effect in config.skill_projection_effects() {
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
		skills::lifetime_definition::LifeTimeDefinition,
		traits::spawn_skill::{
			SkillContact,
			SkillContactEffects,
			SkillLifetime,
			SkillProjection,
			SkillProjectionEffects,
		},
	};
	use bevy::{ecs::bundle::Bundle, math::Vec3};
	use common::{
		components::{lifetime::Lifetime, persistent_entity::PersistentEntity},
		effects::force::Force,
		tools::{Units, action_key::slot::SlotKey},
		traits::handles_skill_physics::{
			Contact,
			ContactShape,
			Effect,
			Motion,
			Projection,
			ProjectionShape,
			Skill,
		},
	};
	use macros::simple_mock;
	use mockall::predicate::eq;
	use std::{collections::HashSet, iter::Copied, slice::Iter, sync::LazyLock, time::Duration};
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

	simple_mock! {
		_Config {}
		impl SkillContact for _Config {
			fn skill_contact(&self, c: SkillCaster, s: SkillSpawner, t: SkillTarget) -> Contact;
		}
		impl SkillContactEffects for _Config {
			type TIter<'a> = Copied<Iter<'a, Effect>> where Self: 'a;
			fn skill_contact_effects<'a>(&'a self) -> Copied<Iter<'a, Effect>>;
		}
		impl SkillProjection for _Config {
			fn skill_projection(&self) -> Projection;
		}
		impl SkillProjectionEffects for _Config {
			type TIter<'a> = Copied<Iter<'a, Effect>> where Self: 'a;
			fn skill_projection_effects<'a>(&'a self) -> Copied<Iter<'a, Effect>>;
		}
		impl SkillLifetime for _Config {
			fn lifetime(&self) -> LifeTimeDefinition;
		}
	}

	const EMPTY: &[Effect] = &[];

	fn contact() -> Contact {
		Contact {
			shape: ContactShape::Sphere {
				radius: Units::from(1.),
				hollow_collider: false,
				destroyed_by: HashSet::from([]),
			},
			motion: Motion::Stationary {
				caster: SkillCaster(PersistentEntity::default()),
				target: SkillTarget::Ground(Vec3::ZERO),
				max_cast_range: Units::from(1.),
			},
		}
	}

	fn projection() -> Projection {
		Projection {
			shape: ProjectionShape::Sphere {
				radius: Units::from(1.),
			},
			offset: None,
		}
	}

	impl Mock_Config {
		fn with_defaults(mut self) -> Self {
			self.expect_skill_contact().return_const(contact());
			self.expect_skill_projection().return_const(projection());
			self.expect_skill_contact_effects()
				.returning(|| EMPTY.iter().copied());
			self.expect_skill_projection_effects()
				.returning(|| EMPTY.iter().copied());
			self.expect_lifetime()
				.return_const(LifeTimeDefinition::<Duration>::Infinite);

			self
		}
	}

	#[test]
	fn spawn_contact_and_projection() {
		static ENTITY: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

		fn contact() -> Contact {
			Contact {
				shape: ContactShape::Sphere {
					radius: Units::from(11.),
					hollow_collider: true,
					destroyed_by: HashSet::from([]),
				},
				motion: Motion::Stationary {
					caster: SkillCaster(*ENTITY),
					target: SkillTarget::Ground(Vec3::ZERO),
					max_cast_range: Units::from(11.),
				},
			}
		}

		fn projection() -> Projection {
			Projection {
				shape: ProjectionShape::Sphere {
					radius: Units::from(1.),
				},
				offset: None,
			}
		}

		let mut spawn = Mock_Spawn::new_mock(assert_contact_and_projection_used);
		let config = Mock_Config::new_mock(|mock| {
			mock.expect_skill_contact().return_const(contact());
			mock.expect_skill_projection().return_const(projection());
		})
		.with_defaults();

		spawn.spawn_skill(
			config,
			SkillCaster(PersistentEntity::default()),
			SkillSpawner::Neutral,
			SkillTarget::Entity(PersistentEntity::default()),
		);

		fn assert_contact_and_projection_used(mock: &mut Mock_Spawn) {
			mock.expect_spawn()
				.once()
				.with(eq(contact()), eq(projection()))
				.returning(|_, _| Mock_Skill::new().with_defaults());
		}
	}

	#[test]
	fn return_stoppable() {
		let root = PersistentEntity::default();
		let mut spawn = Mock_Spawn::new_mock(move |mock| {
			mock.expect_spawn().returning(move |_, _| {
				Mock_Skill::new_mock(|mock| {
					mock.expect_root().return_const(root);
				})
				.with_defaults()
			});
		});
		let config = Mock_Config::new_mock(|mock| {
			mock.expect_lifetime()
				.return_const(LifeTimeDefinition::<Duration>::UntilStopped);
		})
		.with_defaults();

		let on_skill_stop = spawn.spawn_skill(
			config,
			SkillCaster(PersistentEntity::default()),
			SkillSpawner::Slot(SlotKey(42)),
			SkillTarget::Entity(PersistentEntity::default()),
		);

		assert_eq!(OnSkillStop::Stop(root), on_skill_stop);
	}

	#[test_case(LifeTimeDefinition::Infinite; "infinite")]
	#[test_case(LifeTimeDefinition::UntilOutlived(Duration::default()); "with lifetime")]
	fn return_non_stoppable(lifetime: LifeTimeDefinition) {
		let root = PersistentEntity::default();
		let mut spawn = Mock_Spawn::new_mock(move |mock| {
			mock.expect_spawn().returning(move |_, _| {
				Mock_Skill::new_mock(|mock| {
					mock.expect_root().return_const(root);
				})
				.with_defaults()
			});
		});
		let config = Mock_Config::new_mock(|mock| {
			mock.expect_lifetime().return_const(lifetime);
		})
		.with_defaults();

		let on_skill_stop = spawn.spawn_skill(
			config,
			SkillCaster(PersistentEntity::default()),
			SkillSpawner::Slot(SlotKey(42)),
			SkillTarget::Entity(PersistentEntity::default()),
		);

		assert_eq!(OnSkillStop::Ignore, on_skill_stop);
	}

	#[test]
	fn use_contact_parameters() {
		static CASTER: LazyLock<SkillCaster> =
			LazyLock::new(|| SkillCaster(PersistentEntity::default()));
		static TARGET: LazyLock<SkillTarget> =
			LazyLock::new(|| SkillTarget::Entity(PersistentEntity::default()));
		const SPAWNER: SkillSpawner = SkillSpawner::Slot(SlotKey(42));
		let mut spawn = Mock_Spawn::new_mock(|mock| {
			mock.expect_spawn()
				.returning(|_, _| Mock_Skill::new().with_defaults());
		});
		let config = Mock_Config::new_mock(assert_contact_parameters).with_defaults();

		spawn.spawn_skill(config, *CASTER, SPAWNER, *TARGET);

		fn assert_contact_parameters(mock: &mut Mock_Config) {
			mock.expect_skill_contact()
				.once()
				.with(eq(*CASTER), eq(SPAWNER), eq(*TARGET))
				.return_const(contact());
		}
	}

	#[test]
	fn add_contact_effect() {
		const EFFECTS: &[Effect] = &[Effect::Force(Force)];
		let mut spawn = Mock_Spawn::new_mock(|mock| {
			mock.expect_spawn()
				.returning(|_, _| Mock_Skill::new_mock(assert_added_effects).with_defaults());
		});
		let config = Mock_Config::new_mock(|mock| {
			mock.expect_skill_contact_effects()
				.returning(|| EFFECTS.iter().copied());
		})
		.with_defaults();

		spawn.spawn_skill(
			config,
			SkillCaster(PersistentEntity::default()),
			SkillSpawner::Neutral,
			SkillTarget::Entity(PersistentEntity::default()),
		);

		fn assert_added_effects(mock: &mut Mock_Skill) {
			mock.expect_insert_on_contact()
				.once()
				.with(eq(Effect::Force(Force)))
				.return_const(());
		}
	}

	#[test]
	fn add_projection_effect() {
		const EFFECTS: &[Effect] = &[Effect::Force(Force)];
		let mut spawn = Mock_Spawn::new_mock(|mock| {
			mock.expect_spawn()
				.returning(|_, _| Mock_Skill::new_mock(assert_added_effects).with_defaults());
		});
		let config = Mock_Config::new_mock(|mock| {
			mock.expect_skill_projection_effects()
				.returning(|| EFFECTS.iter().copied());
		})
		.with_defaults();

		spawn.spawn_skill(
			config,
			SkillCaster(PersistentEntity::default()),
			SkillSpawner::Neutral,
			SkillTarget::Entity(PersistentEntity::default()),
		);

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
		let config = Mock_Config::new_mock(|mock| {
			mock.expect_lifetime()
				.return_const(LifeTimeDefinition::UntilOutlived(Duration::from_secs(2)));
		})
		.with_defaults();

		spawn.spawn_skill(
			config,
			SkillCaster(PersistentEntity::default()),
			SkillSpawner::Neutral,
			SkillTarget::Entity(PersistentEntity::default()),
		);

		fn assert_added_effects(mock: &mut Mock_Skill) {
			mock.expect_insert_on_root()
				.once()
				.with(eq(Lifetime::from(Duration::from_secs(2))))
				.return_const(());
		}
	}
}
