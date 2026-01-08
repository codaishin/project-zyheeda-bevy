use crate::{
	ActiveSkill,
	skills::behaviors::{SkillBehaviorConfig, dto::SkillBehaviorConfigDto},
};
use common::{
	errors::Unreachable,
	traits::{handles_custom_assets::TryLoadFrom, load_asset::LoadAsset},
};

impl From<ActiveSkill> for ActiveSkill<SkillBehaviorConfigDto> {
	fn from(value: ActiveSkill) -> Self {
		match value {
			ActiveSkill::Idle => Self::Idle,
			ActiveSkill::Start { slot_key, shape } => Self::Start {
				slot_key,
				shape: SkillBehaviorConfigDto::from(shape),
			},
			ActiveSkill::Stoppable(persistent_entity) => Self::Stoppable(persistent_entity),
			ActiveSkill::Stop(persistent_entity) => Self::Stop(persistent_entity),
		}
	}
}

impl TryLoadFrom<ActiveSkill<SkillBehaviorConfigDto>> for ActiveSkill {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		value: ActiveSkill<SkillBehaviorConfigDto>,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		let executer = match value {
			ActiveSkill::Idle => Self::Idle,
			ActiveSkill::Start { slot_key, shape } => Self::Start {
				slot_key,
				shape: SkillBehaviorConfig::from(shape),
			},
			ActiveSkill::Stoppable(persistent_entity) => Self::Stoppable(persistent_entity),
			ActiveSkill::Stop(persistent_entity) => Self::Stop(persistent_entity),
		};

		Ok(executer)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::skills::shape::{SkillShape, shield::Shield};
	use bevy::{asset::AssetPath, prelude::*};
	use common::{
		components::persistent_entity::PersistentEntity,
		tools::action_key::slot::{PlayerSlot, Side, SlotKey},
	};
	use test_case::test_case;

	fn start_spawn_shield() -> ActiveSkill {
		ActiveSkill::Start {
			slot_key: SlotKey::from(PlayerSlot::Lower(Side::Left)),
			shape: SkillBehaviorConfig::from_shape(SkillShape::Shield(Shield)),
		}
	}

	struct _LoadAsset;

	impl LoadAsset for _LoadAsset {
		fn load_asset<'a, TAsset, TPath>(&mut self, _: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'a>>,
		{
			panic!("NOT USED")
		}
	}

	#[test_case(ActiveSkill::Idle; "idle")]
	#[test_case(start_spawn_shield(); "start")]
	#[test_case(ActiveSkill::Stoppable(PersistentEntity::default()); "started stoppable")]
	#[test_case(ActiveSkill::Stop(PersistentEntity::default()); "stop")]
	fn roundtrip_survives(original: ActiveSkill) {
		let dto = ActiveSkill::<SkillBehaviorConfigDto>::from(original.clone());
		let Ok(restored) = ActiveSkill::try_load_from(dto, &mut _LoadAsset);
		assert_eq!(original, restored);
	}
}
