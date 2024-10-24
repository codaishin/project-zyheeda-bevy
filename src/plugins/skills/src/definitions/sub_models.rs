use crate::slot_key::SlotKey;
use bevy::{
	asset::Handle,
	prelude::{Mesh, With},
};
use common::components::{Player, Side};
use items::traits::view::ItemView;
use std::marker::PhantomData;

#[allow(dead_code)] // FIXME: remove "allow" when properly integrated
pub(crate) struct SubModels<T>(PhantomData<T>);

impl ItemView<SlotKey> for SubModels<Player> {
	type TFilter = With<Handle<Mesh>>;
	type TViewComponents = (); // FIXME: Use correct component

	fn view_entity_name(key: &SlotKey) -> &'static str {
		match key {
			SlotKey::TopHand(Side::Left) => "ArmTopLeftModel",
			SlotKey::TopHand(Side::Right) => "ArmTopRightModel",
			SlotKey::BottomHand(Side::Left) => "ArmBottomLeftModel",
			SlotKey::BottomHand(Side::Right) => "ArmBottomRightModel",
		}
	}
}
