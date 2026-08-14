use crate::{skills::shape::OnSkillStop, traits::spawn_skill::SpawnSkillInternal};
use common::prelude::*;

impl<T, TConfig> SpawnSkillInternal<TConfig> for T
where
	T: SpawnSkill,
	TConfig: SkillConfigData,
{
	fn spawn_skill_internal(
		&mut self,
		config: TConfig,
		caster: SkillCaster,
		slot: SlotKey,
	) -> OnSkillStop {
		let skill = self.spawn_skill(SpawnArgs {
			shape: config.shape(),
			mount: config.mount(slot),
			contact_effects: config.contact_effects(),
			projection_effects: config.projection_effects(),
			caster,
		});

		config.on_skill_stop(skill)
	}
}

pub(crate) trait SkillConfigData {
	fn mount(&self, slot: SlotKey) -> SkillMount;
	fn shape(&self) -> &'_ SkillShape;
	fn contact_effects(&self) -> &'_ [SkillEffect];
	fn projection_effects(&self) -> &'_ [SkillEffect];
	fn on_skill_stop(&self, skill: PersistentEntity) -> OnSkillStop;
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::default;
	use macros::simple_mock;
	use std::{sync::LazyLock, time::Duration};
	use test_case::test_case;
	use testing::Mock;

	struct _Config {
		shape: SkillShape,
		contact: Vec<SkillEffect>,
		projection: Vec<SkillEffect>,
		mount: fn(SlotKey) -> SkillMount,
		on_skill_stop: fn(PersistentEntity) -> OnSkillStop,
	}

	impl _Config {
		const DEFAULT: Self = Self {
			shape: SkillShape::SphereAoE(SphereAoE {
				max_range: Units::from_u8(u8::MAX),
				radius: Units::from_u8(1),
				lifetime: Some(Duration::from_secs(1)),
			}),
			contact: vec![],
			projection: vec![],
			mount: |_| SkillMount::Center,
			on_skill_stop: |_| OnSkillStop::Ignore,
		};
	}

	impl Default for _Config {
		fn default() -> Self {
			Self::DEFAULT
		}
	}

	impl SkillConfigData for _Config {
		fn mount(&self, slot: SlotKey) -> SkillMount {
			(self.mount)(slot)
		}

		fn shape(&self) -> &'_ SkillShape {
			&self.shape
		}

		fn contact_effects(&self) -> &'_ [SkillEffect] {
			&self.contact
		}

		fn projection_effects(&self) -> &'_ [SkillEffect] {
			&self.projection
		}

		fn on_skill_stop(&self, skill: PersistentEntity) -> OnSkillStop {
			(self.on_skill_stop)(skill)
		}
	}

	simple_mock! {
		_Spawn {}
		impl SpawnSkill for _Spawn {
			fn spawn_skill<'a>(&'a mut self, args: SpawnArgs<'a>) -> PersistentEntity;
		}
	}

	static CASTER: LazyLock<SkillCaster> =
		LazyLock::new(|| SkillCaster(PersistentEntity::default()));
	const SLOT: SlotKey = SlotKey(123);

	#[test_case(SkillMount::slot; "slot")]
	#[test_case(|_| SkillMount::center(); "center")]
	#[test_case(|_| SkillMount::neutral_slot(); "neutral")]
	fn spawn_on_mount(mount: fn(SlotKey) -> SkillMount) {
		let config = _Config { mount, ..default() };
		let mut spawn = Mock_Spawn::new_mock(assert_mount(mount));

		spawn.spawn_skill_internal(config, *CASTER, SLOT);

		fn assert_mount(mount: fn(SlotKey) -> SkillMount) -> impl FnMut(&mut Mock_Spawn) {
			move |mock| {
				mock.expect_spawn_skill()
					.once()
					.withf(move |args| {
						args == &SpawnArgs {
							shape: &_Config::DEFAULT.shape,
							caster: *CASTER,
							mount: mount(SLOT),
							contact_effects: &[],
							projection_effects: &[],
						}
					})
					.return_const(PersistentEntity::default());
			}
		}
	}

	#[test_case(|_| OnSkillStop::Ignore; "infinite")]
	#[test_case(OnSkillStop::Stop; "with lifetime")]
	fn return_skill_stop(on_skill_stop: fn(PersistentEntity) -> OnSkillStop) {
		let entity = PersistentEntity::default();
		let config = _Config {
			on_skill_stop,
			..default()
		};
		let mut spawn = Mock_Spawn::new_mock(move |mock| {
			mock.expect_spawn_skill().return_const(entity);
		});

		let result = spawn.spawn_skill_internal(config, *CASTER, SLOT);

		assert_eq!(on_skill_stop(entity), result);
	}

	#[test]
	fn add_contact_effect() {
		let config = _Config {
			contact: vec![SkillEffect::Force(Force)],
			..default()
		};
		let mut spawn = Mock_Spawn::new_mock(assert_added_effects);

		spawn.spawn_skill_internal(config, *CASTER, SLOT);

		fn assert_added_effects(mock: &mut Mock_Spawn) {
			mock.expect_spawn_skill()
				.once()
				.withf(|args| args.contact_effects == [SkillEffect::Force(Force)])
				.return_const(PersistentEntity::default());
		}
	}

	#[test]
	fn add_projection_effect() {
		let config = _Config {
			projection: vec![SkillEffect::Force(Force)],
			..default()
		};
		let mut spawn = Mock_Spawn::new_mock(assert_added_effects);

		spawn.spawn_skill_internal(config, *CASTER, SLOT);

		fn assert_added_effects(mock: &mut Mock_Spawn) {
			mock.expect_spawn_skill()
				.once()
				.withf(|args| args.projection_effects == [SkillEffect::Force(Force)])
				.return_const(PersistentEntity::default());
		}
	}
}
