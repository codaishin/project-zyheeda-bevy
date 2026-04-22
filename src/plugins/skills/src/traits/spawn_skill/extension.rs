use crate::{skills::shape::OnSkillStop, traits::spawn_skill::SpawnSkill};
use common::{
	components::persistent_entity::PersistentEntity,
	tools::action_key::slot::SlotKey,
	traits::handles_skill_physics::{
		Effect,
		SkillCaster,
		SkillMount,
		SkillShape,
		Spawn,
		SpawnArgs,
	},
};

impl<T, TConfig> SpawnSkill<TConfig> for T
where
	T: Spawn,
	TConfig: SkillConfigData,
{
	fn spawn_skill(&mut self, config: TConfig, caster: SkillCaster, slot: SlotKey) -> OnSkillStop {
		let mount = if config.use_neutral_mount() {
			SkillMount::NeutralSlot
		} else {
			SkillMount::Slot(slot)
		};

		let skill = self.spawn(SpawnArgs {
			shape: config.shape(),
			contact_effects: config.contact_effects(),
			projection_effects: config.projection_effects(),
			caster,
			mount,
		});

		config.on_skill_stop(skill)
	}
}

pub(crate) trait SkillConfigData {
	fn use_neutral_mount(&self) -> bool;
	fn shape(&self) -> &'_ SkillShape;
	fn contact_effects(&self) -> &'_ [Effect];
	fn projection_effects(&self) -> &'_ [Effect];
	fn on_skill_stop(&self, skill: PersistentEntity) -> OnSkillStop;
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::default;
	use common::{
		components::persistent_entity::PersistentEntity,
		effects::force::Force,
		tools::{Units, action_key::slot::SlotKey},
		traits::handles_skill_physics::{Effect, SkillShape, ground_target::SphereAoE},
	};
	use macros::simple_mock;
	use std::time::Duration;
	use test_case::test_case;
	use testing::Mock;
	use uuid::uuid;

	struct _Config {
		use_neutral_spawn: bool,
		shape: SkillShape,
		contact: Vec<Effect>,
		projection: Vec<Effect>,
		on_skill_stop: fn(PersistentEntity) -> OnSkillStop,
	}

	impl _Config {
		const DEFAULT: Self = Self {
			use_neutral_spawn: false,
			shape: SkillShape::SphereAoE(SphereAoE {
				max_range: Units::from_u8(u8::MAX),
				radius: Units::from_u8(1),
				lifetime: Some(Duration::from_secs(1)),
			}),
			contact: vec![],
			projection: vec![],
			on_skill_stop: |_| OnSkillStop::Ignore,
		};

		const fn using_neutral_spawn(mut self, use_neutral_spawn: bool) -> Self {
			self.use_neutral_spawn = use_neutral_spawn;
			self
		}
	}

	impl Default for _Config {
		fn default() -> Self {
			Self::DEFAULT
		}
	}

	impl SkillConfigData for _Config {
		fn use_neutral_mount(&self) -> bool {
			self.use_neutral_spawn
		}

		fn shape(&self) -> &'_ SkillShape {
			&self.shape
		}

		fn contact_effects(&self) -> &'_ [Effect] {
			&self.contact
		}

		fn projection_effects(&self) -> &'_ [Effect] {
			&self.projection
		}

		fn on_skill_stop(&self, skill: PersistentEntity) -> OnSkillStop {
			(self.on_skill_stop)(skill)
		}
	}

	simple_mock! {
		_Spawn {}
		impl Spawn for _Spawn {
			fn spawn<'a>(&'a mut self, args: SpawnArgs<'a>) -> PersistentEntity;
		}
	}

	static CASTER: SkillCaster = SkillCaster(PersistentEntity::from_uuid(uuid!(
		"91409ebe-e94d-43b2-8dc1-53949e4f0dc2"
	)));
	const SLOT: SlotKey = SlotKey(123);

	#[test]
	fn spawn_contact_and_projection_on_slot() {
		const CONFIG: _Config = _Config::DEFAULT.using_neutral_spawn(false);
		const ARGS: SpawnArgs = SpawnArgs {
			shape: &CONFIG.shape,
			caster: CASTER,
			mount: SkillMount::Slot(SLOT),
			contact_effects: &[],
			projection_effects: &[],
		};
		let mut spawn = Mock_Spawn::new_mock(assert_contact_and_projection_used);

		spawn.spawn_skill(CONFIG, CASTER, SLOT);

		fn assert_contact_and_projection_used(mock: &mut Mock_Spawn) {
			mock.expect_spawn()
				.once()
				.withf(|args| args == &ARGS)
				.return_const(PersistentEntity::default());
		}
	}

	#[test]
	fn spawn_contact_and_projection_on_neutral() {
		const CONFIG: _Config = _Config::DEFAULT.using_neutral_spawn(true);
		const ARGS: SpawnArgs = SpawnArgs {
			shape: &CONFIG.shape,
			caster: CASTER,
			mount: SkillMount::NeutralSlot,
			contact_effects: &[],
			projection_effects: &[],
		};
		let mut spawn = Mock_Spawn::new_mock(assert_contact_and_projection_used);

		spawn.spawn_skill(CONFIG, CASTER, SLOT);

		fn assert_contact_and_projection_used(mock: &mut Mock_Spawn) {
			mock.expect_spawn()
				.once()
				.withf(|args| args == &ARGS)
				.return_const(PersistentEntity::default());
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
			mock.expect_spawn().return_const(entity);
		});

		let result = spawn.spawn_skill(config, CASTER, SLOT);

		assert_eq!(on_skill_stop(entity), result);
	}

	#[test]
	fn add_contact_effect() {
		let config = _Config {
			contact: vec![Effect::Force(Force)],
			..default()
		};
		let mut spawn = Mock_Spawn::new_mock(assert_added_effects);

		spawn.spawn_skill(config, CASTER, SLOT);

		fn assert_added_effects(mock: &mut Mock_Spawn) {
			mock.expect_spawn()
				.once()
				.withf(|args| args.contact_effects == [Effect::Force(Force)])
				.return_const(PersistentEntity::default());
		}
	}

	#[test]
	fn add_projection_effect() {
		let config = _Config {
			projection: vec![Effect::Force(Force)],
			..default()
		};
		let mut spawn = Mock_Spawn::new_mock(assert_added_effects);

		spawn.spawn_skill(config, CASTER, SLOT);

		fn assert_added_effects(mock: &mut Mock_Spawn) {
			mock.expect_spawn()
				.once()
				.withf(|args| args.projection_effects == [Effect::Force(Force)])
				.return_const(PersistentEntity::default());
		}
	}
}
