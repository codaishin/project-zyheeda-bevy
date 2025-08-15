mod dto;

use super::model_render::ModelRender;
use crate::{components::slots::dto::SlotsDto, item::Item, traits::loadout_key::LoadoutKey};
use bevy::{asset::Handle, prelude::*};
use common::{
	components::{asset_model::AssetModel, essence::Essence},
	tools::action_key::slot::{PlayerSlot, Side, SlotKey},
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

impl GetRef<SlotKey> for Slots {
	type TValue<'a>
		= &'a Handle<Item>
	where
		Self: 'a;

	fn get_ref(&self, key: &SlotKey) -> Option<&Handle<Item>> {
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
	type TKey = PlayerSlot;
	type TAsset = Item;

	fn get_asset<'a, TAssets>(
		&'a self,
		key: &Self::TKey,
		assets: &'a TAssets,
	) -> Option<&'a Self::TAsset>
	where
		TAssets: GetRef<Handle<Self::TAsset>, TValue<'a> = &'a Self::TAsset>,
	{
		assets.get_ref(self.get_ref(&SlotKey::from(*key))?)
	}
}

pub(crate) struct HandItemSlots;

impl ChildName<HandItemSlots> for PlayerSlot {
	fn child_name(&self) -> &'static str {
		match self {
			PlayerSlot::Upper(Side::Left) => "top_hand_slot.L",
			PlayerSlot::Upper(Side::Right) => "top_hand_slot.R",
			PlayerSlot::Lower(Side::Left) => "bottom_hand_slot.L",
			PlayerSlot::Lower(Side::Right) => "bottom_hand_slot.R",
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
			}) => AssetModel::path(path),
			_ => AssetModel::none(),
		}
	}
}

impl ChildAssetDefinition<HandItemSlots> for Slots {
	type TChildKey = PlayerSlot;
	type TChildFilter = ();
	type TChildAsset = Item;
}

pub(crate) struct ForearmItemSlots;

impl ChildName<ForearmItemSlots> for PlayerSlot {
	fn child_name(&self) -> &'static str {
		match self {
			PlayerSlot::Upper(Side::Left) => "top_forearm.L",
			PlayerSlot::Upper(Side::Right) => "top_forearm.R",
			PlayerSlot::Lower(Side::Left) => "bottom_forearm.L",
			PlayerSlot::Lower(Side::Right) => "bottom_forearm.R",
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
			}) => AssetModel::path(path),
			_ => AssetModel::none(),
		}
	}
}

impl ChildAssetDefinition<ForearmItemSlots> for Slots {
	type TChildKey = PlayerSlot;
	type TChildFilter = ();
	type TChildAsset = Item;
}

pub(crate) struct SubMeshEssenceSlots;

impl ChildName<SubMeshEssenceSlots> for PlayerSlot {
	fn child_name(&self) -> &'static str {
		match self {
			PlayerSlot::Upper(Side::Left) => "ArmTopLeftData",
			PlayerSlot::Upper(Side::Right) => "ArmTopRightData",
			PlayerSlot::Lower(Side::Left) => "ArmBottomLeftData",
			PlayerSlot::Lower(Side::Right) => "ArmBottomRightData",
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
	type TChildKey = PlayerSlot;
	type TChildFilter = With<Mesh3d>;
	type TChildAsset = Item;
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::new_handle;

	#[test]
	fn get_some() {
		let item = new_handle();
		let slots = Slots([(SlotKey(2), Some(item.clone()))].into());

		assert_eq!(Some(&item), slots.get_ref(&SlotKey(2)));
	}

	#[test]
	fn get_none() {
		let slots = Slots([(SlotKey(7), Some(new_handle()))].into());

		assert_eq!(None::<&Handle<Item>>, slots.get_ref(&SlotKey(11)));
	}
}
