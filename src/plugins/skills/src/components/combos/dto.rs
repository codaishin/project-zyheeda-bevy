use crate::components::{
	combo_node::{ComboNode, dto::ComboNodeDto},
	combos::Combos,
};
use bevy::prelude::*;
use common::{
	errors::Unreachable,
	traits::{handles_custom_assets::TryLoadFrom, load_asset::LoadAsset},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CombosDto {
	config: ComboNodeDto,
	current: Option<ComboNodeDto>,
}

impl From<Combos> for CombosDto {
	fn from(Combos { config, current }: Combos) -> Self {
		Self {
			config: ComboNodeDto::from(config),
			current: current.map(ComboNodeDto::from),
		}
	}
}

impl TryLoadFrom<CombosDto> for Combos {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		CombosDto { config, current }: CombosDto,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		let Ok(config) = ComboNode::try_load_from(config, asset_server);
		let current = match current {
			Some(current) => {
				let Ok(current) = ComboNode::try_load_from(current, asset_server);
				Some(current)
			}
			None => None,
		};

		Ok(Self { config, current })
	}
}
