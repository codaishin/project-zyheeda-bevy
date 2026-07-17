use crate::{
	assets::door_meta::DoorMeta,
	components::{door_meta_handle::DoorMetaHandle, interactive::Interactive},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	components::{
		model::{Model, SceneId, UseGltfLookup},
		persistent_entity::PersistentEntity,
	},
	errors::Unreachable,
	systems::register_animations::AnimationsMarker,
	traits::{
		handles_map_generation::InteractiveType,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use macros::{SavableComponent, asset_path};
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[savable_component(id = "door")]
#[component(immutable)]
#[require(
	PersistentEntity,
	Interactive { interactive_type: InteractiveType::Door },
	DoorMetaHandle,
	Transform,
	ApplyDoorAnimations,
	ApplyDoorFrame
)]
pub(crate) struct Door;

impl Prefab<()> for Door {
	type TError = Unreachable;
	type TSystemParam = Res<'static, AssetServer>;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		assets: StaticSystemParam<Res<AssetServer>>,
	) -> Result<(), Self::TError> {
		entity.try_insert((
			Name::from("SlideDoor"),
			Model::scene((
				asset_path!("maps/assets/slide_door/model.glb"),
				SceneId(0),
				UseGltfLookup(true),
			)),
			DoorMetaHandle(assets.load(asset_path!("maps/assets/slide_door/meta.door"))),
		));

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct ApplyDoorAnimations;

impl AnimationsMarker for ApplyDoorAnimations {
	type TConfig = DoorMeta;
	type TConfigComponent = DoorMetaHandle;
}

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct ApplyDoorFrame;
