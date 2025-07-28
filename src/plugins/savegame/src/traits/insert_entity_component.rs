use bevy::prelude::*;
use common::traits::load_asset::LoadAsset;

pub(crate) trait InsertEntityComponent<TLoadAsset>
where
	TLoadAsset: LoadAsset,
{
	type TComponent;
	type TError;

	fn component_name(&self) -> &'static str;
	fn insert_component(
		&self,
		entity: &mut EntityCommands,
		components: Self::TComponent,
		asset_server: &mut TLoadAsset,
	) -> Result<(), Self::TError>;
}
