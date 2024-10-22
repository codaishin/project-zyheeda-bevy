use crate::slot_key::SlotKey;
use common::components::{Player, Side};
use items::traits::{entity_names::EntityNames, key_string::KeyString};
use std::marker::PhantomData;

const TOP_HAND_L: &str = "top_hand_slot.L";
const TOP_HAND_R: &str = "top_hand_slot.R";
const BTM_HAND_L: &str = "bottom_hand_slot.L";
const BTM_HAND_R: &str = "bottom_hand_slot.R";

const TOP_FOREARM_L: &str = "top_forearm.L";
const TOP_FOREARM_R: &str = "top_forearm.R";
const BTM_FOREARM_L: &str = "bottom_forearm.L";
const BTM_FOREARM_R: &str = "bottom_forearm.R";

#[derive(Debug, PartialEq)]
pub struct HandSlots<T>(PhantomData<T>);

impl EntityNames for HandSlots<Player> {
	fn entity_names() -> Vec<&'static str> {
		vec![TOP_HAND_L, TOP_HAND_R, BTM_HAND_L, BTM_HAND_R]
	}
}

impl KeyString<SlotKey> for HandSlots<Player> {
	fn key_string(key: &SlotKey) -> &'static str {
		match key {
			SlotKey::TopHand(Side::Left) => TOP_HAND_L,
			SlotKey::TopHand(Side::Right) => TOP_HAND_R,
			SlotKey::BottomHand(Side::Left) => BTM_HAND_L,
			SlotKey::BottomHand(Side::Right) => BTM_HAND_R,
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct ForearmSlots<T>(PhantomData<T>);

impl EntityNames for ForearmSlots<Player> {
	fn entity_names() -> Vec<&'static str> {
		vec![TOP_FOREARM_L, TOP_FOREARM_R, BTM_FOREARM_L, BTM_FOREARM_R]
	}
}

impl KeyString<SlotKey> for ForearmSlots<Player> {
	fn key_string(key: &SlotKey) -> &'static str {
		match key {
			SlotKey::TopHand(Side::Left) => TOP_FOREARM_L,
			SlotKey::TopHand(Side::Right) => TOP_FOREARM_R,
			SlotKey::BottomHand(Side::Left) => BTM_FOREARM_L,
			SlotKey::BottomHand(Side::Right) => BTM_FOREARM_R,
		}
	}
}
