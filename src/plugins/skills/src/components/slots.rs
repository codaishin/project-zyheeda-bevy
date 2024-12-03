use super::model_render::ModelRender;
use crate::{item::Item, slot_key::SlotKey};
use bevy::prelude::*;
use common::{
	components::{essence::Essence, AssetModel, Side},
	traits::{
		accessors::get::GetRef,
		get_asset::GetAsset,
		register_item_visualization::{ContainsVisibleItemAssets, Forearm, Hand, SubMeshEssence},
	},
};
use std::{collections::HashMap, fmt::Debug};

#[derive(Component, Clone, PartialEq, Debug)]
pub struct Slots(pub HashMap<SlotKey, Option<Handle<Item>>>);

impl Slots {
	pub fn new<const N: usize>(slots: [(SlotKey, Option<Handle<Item>>); N]) -> Self {
		Self(HashMap::from(slots))
	}
}

impl Default for Slots {
	fn default() -> Self {
		Self::new([])
	}
}

impl GetRef<SlotKey, Handle<Item>> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&Handle<Item>> {
		let slot = self.0.get(key)?;
		slot.as_ref()
	}
}

impl GetAsset for Slots {
	type TKey = SlotKey;
	type TAsset = Item;

	fn get_asset<'a, TAssets>(
		&'a self,
		key: &Self::TKey,
		assets: &'a TAssets,
	) -> Option<&'a Self::TAsset>
	where
		TAssets: GetRef<Handle<Self::TAsset>, Self::TAsset>,
	{
		assets.get(self.get(key)?)
	}
}

impl ContainsVisibleItemAssets<Hand> for Slots {
	type TVisualizationEntityConstraint = ();

	fn visualization_entity_name(key: &Self::TKey) -> &'static str {
		match key {
			SlotKey::TopHand(Side::Left) => "top_hand_slot.L",
			SlotKey::TopHand(Side::Right) => "top_hand_slot.R",
			SlotKey::BottomHand(Side::Left) => "bottom_hand_slot.L",
			SlotKey::BottomHand(Side::Right) => "bottom_hand_slot.R",
		}
	}

	fn visualization_component(item: Option<&Self::TAsset>) -> impl Bundle {
		match item {
			Some(Item {
				model: ModelRender::Hand(asset_model),
				..
			}) => asset_model.clone(),
			_ => AssetModel::None,
		}
	}
}

impl ContainsVisibleItemAssets<Forearm> for Slots {
	type TVisualizationEntityConstraint = ();

	fn visualization_entity_name(key: &Self::TKey) -> &'static str {
		match key {
			SlotKey::TopHand(Side::Left) => "top_forearm.L",
			SlotKey::TopHand(Side::Right) => "top_forearm.R",
			SlotKey::BottomHand(Side::Left) => "bottom_forearm.L",
			SlotKey::BottomHand(Side::Right) => "bottom_forearm.R",
		}
	}

	fn visualization_component(item: Option<&Self::TAsset>) -> impl Bundle {
		match item {
			Some(Item {
				model: ModelRender::Forearm(asset_model),
				..
			}) => asset_model.clone(),
			_ => AssetModel::None,
		}
	}
}

impl ContainsVisibleItemAssets<SubMeshEssence> for Slots {
	type TVisualizationEntityConstraint = With<Handle<Mesh>>;

	fn visualization_entity_name(key: &Self::TKey) -> &'static str {
		match key {
			SlotKey::TopHand(Side::Left) => "ArmTopLeftModel",
			SlotKey::TopHand(Side::Right) => "ArmTopRightModel",
			SlotKey::BottomHand(Side::Left) => "ArmBottomLeftModel",
			SlotKey::BottomHand(Side::Right) => "ArmBottomRightModel",
		}
	}

	fn visualization_component(item: Option<&Self::TAsset>) -> impl Bundle {
		match item {
			Some(Item { essence, .. }) => *essence,
			_ => Essence::None,
		};
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{components::Side, test_tools::utils::new_handle};

	#[test]
	fn get_off_hand() {
		let item = new_handle();
		let slots = Slots([(SlotKey::BottomHand(Side::Left), Some(item.clone()))].into());

		assert_eq!(Some(&item), slots.get(&SlotKey::BottomHand(Side::Left)));
	}

	#[test]
	fn get_main_hand() {
		let item = new_handle();
		let slots = Slots([(SlotKey::BottomHand(Side::Right), Some(item.clone()))].into());

		assert_eq!(Some(&item), slots.get(&SlotKey::BottomHand(Side::Right)));
	}

	#[test]
	fn get_none() {
		let slots = Slots([(SlotKey::BottomHand(Side::Right), Some(new_handle()))].into());

		assert_eq!(
			None::<&Handle<Item>>,
			slots.get(&SlotKey::BottomHand(Side::Left))
		);
	}
}
