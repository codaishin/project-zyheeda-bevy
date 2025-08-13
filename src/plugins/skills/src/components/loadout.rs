use crate::components::{
	combos::Combos,
	combos_time_out::CombosTimeOut,
	inventory::Inventory,
	queue::Queue,
	skill_executer::SkillExecuter,
	slots::Slots,
	swapper::Swapper,
};
use bevy::prelude::*;
use common::{
	errors::Error,
	tools::action_key::slot::{PlayerSlot, Side, SlotKey},
	traits::{
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use macros::item_asset;
use std::time::Duration;

#[derive(Component, Debug, Default)]
#[require(
	Combos,
	CombosTimeOut = CombosTimeOut::after(Duration::from_secs(2)),
	Queue,
	SkillExecuter,
	Swapper,
)]
pub(crate) struct Loadout;

impl Prefab<()> for Loadout {
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		assets: &mut impl LoadAsset,
	) -> Result<(), Error> {
		entity.try_insert_if_new((
			Inventory::from([
				Some(assets.load_asset(item_asset!("pistol"))),
				Some(assets.load_asset(item_asset!("pistol"))),
				Some(assets.load_asset(item_asset!("pistol"))),
			]),
			Slots::from([
				(
					SlotKey::from(PlayerSlot::Upper(Side::Left)),
					Some(assets.load_asset(item_asset!("pistol"))),
				),
				(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
					Some(assets.load_asset(item_asset!("pistol"))),
				),
				(
					SlotKey::from(PlayerSlot::Lower(Side::Right)),
					Some(assets.load_asset(item_asset!("force_essence"))),
				),
				(
					SlotKey::from(PlayerSlot::Upper(Side::Right)),
					Some(assets.load_asset(item_asset!("force_essence"))),
				),
			]),
		));
		Ok(())
	}
}
