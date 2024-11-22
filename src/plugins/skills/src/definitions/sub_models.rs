use crate::slot_key::SlotKey;
use bevy::{
	asset::Handle,
	prelude::{Mesh, With},
};
use common::components::{essence::Essence, Side};
use items::traits::view::ItemView;
use player::components::player::Player;
use std::marker::PhantomData;

pub(crate) struct SubModels<T>(PhantomData<T>);

impl ItemView<SlotKey> for SubModels<Player> {
	type TFilter = With<Handle<Mesh>>;
	type TViewComponents = Essence;

	fn view_entity_name(key: &SlotKey) -> &'static str {
		match key {
			SlotKey::TopHand(Side::Left) => "ArmTopLeftModel",
			SlotKey::TopHand(Side::Right) => "ArmTopRightModel",
			SlotKey::BottomHand(Side::Left) => "ArmBottomLeftModel",
			SlotKey::BottomHand(Side::Right) => "ArmBottomRightModel",
		}
	}
}
