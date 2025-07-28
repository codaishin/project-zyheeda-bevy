use crate::{context::EntityLoadBuffer, errors::SerdeJsonError};
use bevy::prelude::*;
use common::traits::load_asset::LoadAsset;

pub(crate) trait InsertEntityComponent<TLoadAsset>
where
	TLoadAsset: LoadAsset,
{
	fn insert_component(
		&self,
		entity: &mut EntityCommands,
		components: &mut EntityLoadBuffer,
		asset_server: &mut TLoadAsset,
	) -> Result<(), SerdeJsonError>;
}
