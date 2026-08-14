use bevy::prelude::*;
use common::prelude::*;

pub(crate) trait InsertEntityComponent<TLoadAsset>
where
	TLoadAsset: LoadAsset,
{
	type TComponent;
	type TError;

	fn id(&self) -> &UniqueComponentId;
	fn insert_component(
		&self,
		entity: &mut EntityCommands,
		components: Self::TComponent,
		asset_server: &mut TLoadAsset,
	) -> Result<(), Self::TError>;
}
