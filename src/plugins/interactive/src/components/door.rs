use crate::{assets::door_meta::DoorMeta, components::door_meta_handle::DoorMetaHandle};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	errors::Unreachable,
	systems::register_animations::AnimationsMarker,
	traits::{
		handles_map_generation::DoorType,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use macros::asset_path;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
#[require(DoorMetaHandle, ApplyDoorAnimations, Transform)]
pub(crate) struct Door(pub(crate) DoorType);

impl Prefab<()> for Door {
	type TError = Unreachable;
	type TSystemParam<'w, 's> = Res<'w, AssetServer>;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		assets: StaticSystemParam<Res<AssetServer>>,
	) -> Result<(), Self::TError> {
		let bundle = match self.0 {
			DoorType::SlideDoor => (
				Name::from("SlideDoor"),
				DoorMetaHandle(assets.load(asset_path!("maps/assets/slide_door/meta.door"))),
			),
		};

		entity.try_insert(bundle);

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct ApplyDoorAnimations;

impl AnimationsMarker for ApplyDoorAnimations {
	type TConfig = DoorMeta;
	type TConfigComponent = DoorMetaHandle;
}
