use crate::{
	items::slot_key::SlotKey,
	traits::{entity_names::EntityNames, key_string::KeyString},
};
use common::components::{Player, Side};
use std::marker::PhantomData;

const ARM_TOP_L: &str = "ArmTopLeftModel";
const ARM_TOP_R: &str = "ArmTopRightModel";
const ARM_BTM_L: &str = "ArmBottomLeftModel";
const ARM_BTM_R: &str = "ArmBottomRightModel";

pub(crate) struct SubModels<T>(PhantomData<T>);

impl EntityNames for SubModels<Player> {
	fn entity_names() -> Vec<&'static str> {
		vec![ARM_TOP_L, ARM_TOP_R, ARM_BTM_L, ARM_BTM_R]
	}
}

impl KeyString<SlotKey> for SubModels<Player> {
	fn key_string(key: &SlotKey) -> &'static str {
		match key {
			SlotKey::TopHand(Side::Left) => ARM_TOP_L,
			SlotKey::TopHand(Side::Right) => ARM_TOP_R,
			SlotKey::BottomHand(Side::Left) => ARM_BTM_L,
			SlotKey::BottomHand(Side::Right) => ARM_BTM_R,
		}
	}
}
