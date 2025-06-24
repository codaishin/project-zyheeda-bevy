use crate::components::slots::Slots;
use common::{
	errors::Unreachable,
	tools::action_key::slot::SlotKey,
	traits::{handles_custom_assets::TryLoadFrom, load_asset::LoadAsset},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SlotsDto(Vec<(SlotKey, Option<String>)>);

impl From<Slots> for SlotsDto {
	fn from(Slots(items): Slots) -> Self {
		Self(
			items
				.into_iter()
				.map(|(key, item)| (key, item.and_then(|item| Some(item.path()?.to_string()))))
				.collect(),
		)
	}
}

impl TryLoadFrom<SlotsDto> for Slots {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		SlotsDto(items): SlotsDto,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		Ok(Self(
			items
				.into_iter()
				.map(|(key, item)| (key, item.map(|item| asset_server.load_asset(item))))
				.collect(),
		))
	}
}
