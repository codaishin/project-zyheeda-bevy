use crate::{
	SkillDto,
	components::{combo_node::ComboNode, combos::Combos},
};
use bevy::prelude::*;
use common::tools::action_key::slot::SlotKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct CombosDto {
	config: ComboNodeDto,
	current: Option<ComboNodeDto>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ComboNodeDto(Vec<(SlotKey, (SkillDto, ComboNodeDto))>);

impl From<Combos> for CombosDto {
	fn from(Combos { config, current }: Combos) -> Self {
		Self {
			config: ComboNodeDto::from(config),
			current: current.map(ComboNodeDto::from),
		}
	}
}

impl From<ComboNode> for ComboNodeDto {
	fn from(node: ComboNode) -> Self {
		Self(
			node.into_iter()
				.map(|(slot_key, (skill, node))| {
					(slot_key, (SkillDto::from(skill), Self::from(node)))
				})
				.collect(),
		)
	}
}
