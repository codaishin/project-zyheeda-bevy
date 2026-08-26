use crate::components::{
	combo_node::{ComboNode, dto::ComboNodeDto},
	combos::CombosInternal,
};
use bevy::prelude::*;
use common::prelude::*;
use macros::serde_model;

#[serde_model]
#[derive(Debug, PartialEq)]
pub struct CombosInternalDto {
	config: ComboNodeDto,
	#[serde(skip_serializing_if = "Option::is_none")]
	current: Option<ComboNodeDto>,
}

impl From<CombosInternal> for CombosInternalDto {
	fn from(CombosInternal { config, current }: CombosInternal) -> Self {
		Self {
			config: ComboNodeDto::from(config),
			current: current.map(ComboNodeDto::from),
		}
	}
}

impl TryLoadFrom<CombosInternalDto> for CombosInternal {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		CombosInternalDto { config, current }: CombosInternalDto,
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
