use super::{handles_load_tracking::LoadGroup, load_asset::Path};
use bevy::prelude::*;

pub trait HandlesAssetResourceLoading {
	fn load_resource_from_assets<TResource, TLoadGroup>(app: &mut App, path: Path)
	where
		TResource: Resource + Asset,
		TLoadGroup: LoadGroup;
}
