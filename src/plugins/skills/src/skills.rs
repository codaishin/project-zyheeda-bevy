pub(crate) mod behaviors;
pub(crate) mod dto;
pub(crate) mod shape;

use crate::{skills::behaviors::SkillBehaviorConfig, traits::ReleaseSkill};
use bevy::prelude::*;
use common::{
	tools::{
		action_key::slot::SlotKey,
		item_type::{CompatibleItems, ItemType},
		path::Path,
	},
	traits::{
		accessors::get::View,
		handles_animations::SkillAnimation,
		handles_custom_assets::AssetFolderPath,
		handles_loadout::skills::{GetSkillId, SkillIcon, SkillToken},
		handles_localization::Token,
	},
};
use serde::{Deserialize, Serialize};
use std::{
	collections::HashSet,
	fmt::{Display, Formatter, Result as FmtResult},
	time::Duration,
};
use uuid::Uuid;

#[cfg(test)]
use uuid::uuid;

#[derive(PartialEq, Debug, Clone, TypePath, Asset)]
#[cfg_attr(test, derive(Default))]
pub struct Skill {
	pub(crate) id: SkillId,
	pub(crate) token: Token,
	pub(crate) cast_time: Duration,
	pub(crate) animation: Option<SkillAnimation>,
	pub(crate) behavior: RunSkillBehavior,
	pub(crate) compatible_items: CompatibleItems,
	pub(crate) icon: Handle<Image>,
}

impl Display for Skill {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match &*self.token {
			"" => write!(f, "Skill(<no token>)"),
			name => write!(f, "Skill({name})"),
		}
	}
}

impl AssetFolderPath for Skill {
	fn asset_folder_path() -> Path {
		Path::from("skills")
	}
}

impl View<SkillToken> for Skill {
	fn view(&self) -> &Token {
		&self.token
	}
}

impl View<SkillIcon> for Skill {
	fn view(&self) -> &Handle<Image> {
		&self.icon
	}
}

impl View<CompatibleItems> for Skill {
	fn view(&self) -> &HashSet<ItemType> {
		&self.compatible_items.0
	}
}

impl GetSkillId<SkillId> for Skill {
	fn get_skill_id(&self) -> SkillId {
		self.id
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SkillId(pub(crate) Uuid);

#[cfg(test)]
impl SkillId {
	const DEFAULT_ID: SkillId = SkillId(uuid!("9443883c-3972-43da-a2d7-0a013f16d564"));
}

#[cfg(test)]
impl Default for SkillId {
	fn default() -> Self {
		Self::DEFAULT_ID
	}
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub enum SkillMode {
	#[default]
	Hold,
	Release,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(test, derive(Default))]
pub struct QueuedSkill {
	pub skill: Skill,
	pub key: SlotKey,
	pub skill_mode: SkillMode,
}

impl QueuedSkill {
	pub(crate) fn new(skill: Skill, key: SlotKey) -> Self {
		Self {
			skill,
			key,
			skill_mode: SkillMode::Hold,
		}
	}
}

impl View<SlotKey> for QueuedSkill {
	fn view(&self) -> SlotKey {
		self.key
	}
}

impl View<Token> for QueuedSkill {
	fn view(&self) -> &Token {
		&self.skill.token
	}
}

impl ReleaseSkill for QueuedSkill {
	fn release_skill(&mut self) {
		self.skill_mode = SkillMode::Release;
	}
}

#[cfg(test)]
mod test_queued {
	use super::*;

	#[test]
	fn prime_skill() {
		let mut queued = QueuedSkill {
			skill: Skill::default(),
			skill_mode: SkillMode::Hold,
			..default()
		};
		queued.release_skill();

		assert_eq!(SkillMode::Release, queued.skill_mode);
	}
}

#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash)]
pub(crate) enum SkillState {
	Aim,
	Active,
}

#[derive(PartialEq, Debug, Clone)]
pub enum RunSkillBehavior {
	OnActive(SkillBehaviorConfig),
	OnAim(SkillBehaviorConfig),
}

#[cfg(test)]
impl Default for RunSkillBehavior {
	fn default() -> Self {
		use common::traits::handles_skill_physics::{SkillShape, shield::Shield};

		Self::OnActive(SkillBehaviorConfig::from_shape(SkillShape::Shield(Shield)))
	}
}
