use crate::slot_key::SlotKey;
use common::components::{AssetModel, Side};
use items::traits::view::ItemView;
use player::components::player::Player;
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub struct HandSlots<T>(PhantomData<T>);

impl ItemView<SlotKey> for HandSlots<Player> {
	type TFilter = ();
	type TViewComponents = AssetModel;

	fn view_entity_name(key: &SlotKey) -> &'static str {
		match key {
			SlotKey::TopHand(Side::Left) => "top_hand_slot.L",
			SlotKey::TopHand(Side::Right) => "top_hand_slot.R",
			SlotKey::BottomHand(Side::Left) => "bottom_hand_slot.L",
			SlotKey::BottomHand(Side::Right) => "bottom_hand_slot.R",
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct ForearmSlots<T>(PhantomData<T>);

impl ItemView<SlotKey> for ForearmSlots<Player> {
	type TFilter = ();
	type TViewComponents = AssetModel;

	fn view_entity_name(key: &SlotKey) -> &'static str {
		match key {
			SlotKey::TopHand(Side::Left) => "top_forearm.L",
			SlotKey::TopHand(Side::Right) => "top_forearm.R",
			SlotKey::BottomHand(Side::Left) => "bottom_forearm.L",
			SlotKey::BottomHand(Side::Right) => "bottom_forearm.R",
		}
	}
}
