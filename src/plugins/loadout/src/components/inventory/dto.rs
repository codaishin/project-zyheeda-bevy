use crate::components::inventory::Inventory;
use bevy::prelude::*;
use common::{
	errors::Unreachable,
	traits::{handles_custom_assets::TryLoadFrom, load_asset::LoadAsset},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InventoryDto(Vec<(usize, String)>);

impl From<Inventory> for InventoryDto {
	fn from(Inventory(items): Inventory) -> Self {
		let mut dto_items = vec![];

		for (index, item) in items.into_iter().enumerate() {
			let Some(item) = item else {
				continue;
			};
			let Some(path) = item.path() else {
				continue;
			};

			dto_items.push((index, path.to_string()));
		}

		Self(dto_items)
	}
}

impl TryLoadFrom<InventoryDto> for Inventory {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		InventoryDto(mut dto_items): InventoryDto,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		dto_items.sort_by(|(a, _), (b, _)| b.cmp(a));
		let mut dto_items = dto_items.into_iter();

		let Some((highest_index, last_item)) = dto_items.next() else {
			return Ok(Self(vec![]));
		};

		let mut items = vec![None; highest_index];
		items.push(Some(asset_server.load_asset(last_item)));

		for (index, item) in dto_items {
			items[index] = Some(asset_server.load_asset(item));
		}

		Ok(Self(items))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::load_asset::mock::MockAssetServer;
	use testing::new_handle;

	#[test]
	fn deserialize_empty() {
		let dto = InventoryDto(vec![]);
		let mut server = MockAssetServer::default();

		let Ok(item) = Inventory::try_load_from(dto, &mut server);

		assert_eq!(Inventory(vec![]), item);
	}

	#[test]
	fn deserialize() {
		let handle_2 = new_handle();
		let handle_5 = new_handle();
		let dto = InventoryDto(vec![
			(2, "asset/path/2".to_owned()),
			(5, "asset/path/5".to_owned()),
		]);
		let mut server = MockAssetServer::default()
			.path("asset/path/2")
			.returns(handle_2.clone())
			.path("asset/path/5")
			.returns(handle_5.clone());

		let Ok(item) = Inventory::try_load_from(dto, &mut server);

		assert_eq!(
			Inventory(vec![None, None, Some(handle_2), None, None, Some(handle_5)]),
			item
		);
	}
}
