use bevy::prelude::*;
use common::{
	tools::{action_key::slot::SlotKey, bone_name::BoneName},
	traits::{
		bone_key::BoneKey,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct BoneDefinitions {
	pub(crate) forearms: HashMap<BoneName, SlotKey>,
	pub(crate) hands: HashMap<BoneName, SlotKey>,
	pub(crate) essences: HashMap<BoneName, SlotKey>,
}

impl BoneKey<ForearmSlot> for BoneDefinitions {
	fn bone_key(&self, value: &str) -> Option<ForearmSlot> {
		self.forearms.get(value).copied().map(ForearmSlot)
	}
}

impl BoneKey<HandSlot> for BoneDefinitions {
	fn bone_key(&self, value: &str) -> Option<HandSlot> {
		self.hands.get(value).copied().map(HandSlot)
	}
}

impl BoneKey<EssenceSlot> for BoneDefinitions {
	fn bone_key(&self, value: &str) -> Option<EssenceSlot> {
		self.essences.get(value).copied().map(EssenceSlot)
	}
}
