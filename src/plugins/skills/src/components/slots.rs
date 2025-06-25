mod dto;

use super::model_render::ModelRender;
use crate::{components::slots::dto::SlotsDto, item::Item, traits::loadout_key::LoadoutKey};
use bevy::{asset::Handle, prelude::*};
use common::{
	components::{asset_model::AssetModel, essence::Essence},
	tools::action_key::slot::{Side, SlotKey},
	traits::{
		accessors::get::GetRef,
		get_asset::GetAsset,
		handles_assets_for_children::{ChildAssetComponent, ChildAssetDefinition, ChildName},
		iterate::Iterate,
	},
};
use macros::SavableComponent;
use std::{collections::HashMap, fmt::Debug};

#[derive(Component, SavableComponent, PartialEq, Debug, Clone)]
#[savable_component(dto = SlotsDto)]
pub struct Slots(pub HashMap<SlotKey, Option<Handle<Item>>>);

impl<T> From<T> for Slots
where
	T: IntoIterator<Item = (SlotKey, Option<Handle<Item>>)>,
{
	fn from(slots: T) -> Self {
		Self(HashMap::from_iter(slots))
	}
}

impl Default for Slots {
	fn default() -> Self {
		Self::from([])
	}
}

impl GetRef<SlotKey, Handle<Item>> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&Handle<Item>> {
		let slot = self.0.get(key)?;
		slot.as_ref()
	}
}

impl LoadoutKey for Slots {
	type TKey = SlotKey;
}

impl<'a> Iterate<'a> for Slots {
	type TItem = (SlotKey, &'a Option<Handle<Item>>);
	type TIter = Iter<'a>;

	fn iterate(&'a self) -> Self::TIter {
		Iter { it: self.0.iter() }
	}
}

pub struct Iter<'a> {
	it: std::collections::hash_map::Iter<'a, SlotKey, Option<Handle<Item>>>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = (SlotKey, &'a Option<Handle<Item>>);

	fn next(&mut self) -> Option<Self::Item> {
		let (key, slot) = self.it.next()?;
		Some((*key, slot))
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

pub(crate) struct HandItemSlots;

impl ChildName<HandItemSlots> for SlotKey {
	fn child_name(&self) -> &'static str {
		match self {
			SlotKey::TopHand(Side::Left) => "top_hand_slot.L",
			SlotKey::TopHand(Side::Right) => "top_hand_slot.R",
			SlotKey::BottomHand(Side::Left) => "bottom_hand_slot.L",
			SlotKey::BottomHand(Side::Right) => "bottom_hand_slot.R",
		}
	}
}

impl ChildAssetComponent<HandItemSlots> for Item {
	type TComponent = AssetModel;

	fn component(item: Option<&Self>) -> Self::TComponent {
		match item {
			Some(Item {
				model: ModelRender::Hand(path),
				..
			}) => AssetModel::Path(path.clone()),
			_ => AssetModel::None,
		}
	}
}

impl ChildAssetDefinition<HandItemSlots> for Slots {
	type TChildKey = SlotKey;
	type TChildFilter = ();
	type TChildAsset = Item;
}

pub(crate) struct ForearmItemSlots;

impl ChildName<ForearmItemSlots> for SlotKey {
	fn child_name(&self) -> &'static str {
		match self {
			SlotKey::TopHand(Side::Left) => "top_forearm.L",
			SlotKey::TopHand(Side::Right) => "top_forearm.R",
			SlotKey::BottomHand(Side::Left) => "bottom_forearm.L",
			SlotKey::BottomHand(Side::Right) => "bottom_forearm.R",
		}
	}
}

impl ChildAssetComponent<ForearmItemSlots> for Item {
	type TComponent = AssetModel;

	fn component(item: Option<&Self>) -> Self::TComponent {
		match item {
			Some(Item {
				model: ModelRender::Forearm(path),
				..
			}) => AssetModel::Path(path.clone()),
			_ => AssetModel::None,
		}
	}
}

impl ChildAssetDefinition<ForearmItemSlots> for Slots {
	type TChildKey = SlotKey;
	type TChildFilter = ();
	type TChildAsset = Item;
}

pub(crate) struct SubMeshEssenceSlots;

impl ChildName<SubMeshEssenceSlots> for SlotKey {
	fn child_name(&self) -> &'static str {
		match self {
			SlotKey::TopHand(Side::Left) => "ArmTopLeftData",
			SlotKey::TopHand(Side::Right) => "ArmTopRightData",
			SlotKey::BottomHand(Side::Left) => "ArmBottomLeftData",
			SlotKey::BottomHand(Side::Right) => "ArmBottomRightData",
		}
	}
}

impl ChildAssetComponent<SubMeshEssenceSlots> for Item {
	type TComponent = Essence;

	fn component(item: Option<&Self>) -> Self::TComponent {
		match item {
			Some(Item { essence, .. }) => *essence,
			_ => Essence::None,
		}
	}
}

impl ChildAssetDefinition<SubMeshEssenceSlots> for Slots {
	type TChildKey = SlotKey;
	type TChildFilter = With<Mesh3d>;
	type TChildAsset = Item;
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::new_handle;

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
