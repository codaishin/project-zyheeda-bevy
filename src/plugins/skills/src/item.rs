pub mod item_type;

pub(crate) mod dto;

use crate::{components::model_render::ModelRender, skills::Skill};
use bevy::prelude::*;
use common::components::essence::Essence;
use item_type::SkillItemType;

#[derive(Debug, PartialEq, Default, Clone, Asset, TypePath)]
pub struct Item {
	pub name: String,
	pub model: ModelRender,
	pub essence: Essence,
	pub skill: Option<Handle<Skill>>,
	pub item_type: SkillItemType,
}

impl Item {
	pub fn named(name: &str) -> Self {
		Self {
			name: name.to_owned(),
			..default()
		}
	}
}
