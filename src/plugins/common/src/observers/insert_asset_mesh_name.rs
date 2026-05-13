use crate::{
	components::asset_mesh_name::AssetMeshName,
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};
use bevy::{gltf::GltfMeshName, prelude::*};
use zyheeda_core::prelude::NormalizedName;

impl AssetMeshName {
	pub(crate) fn insert(
		inserted_name: On<Insert, GltfMeshName>,
		names: Query<&GltfMeshName>,
		mut commands: ZyheedaCommands,
	) {
		let Ok(GltfMeshName(name)) = names.get(inserted_name.entity) else {
			return;
		};

		commands.try_apply_on(&inserted_name.entity, |mut e| {
			e.try_insert(AssetMeshName(NormalizedName::from(name.clone())));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;
	use zyheeda_core::prelude::NormalizedName;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(AssetMeshName::insert);

		app
	}

	#[test]
	fn insert_asset_mesh_name() {
		let mut app = setup();

		let entity = app.world_mut().spawn(GltfMeshName("name".to_owned()));

		assert_eq!(
			Some(&AssetMeshName(NormalizedName::from("name".to_owned()))),
			entity.get::<AssetMeshName>(),
		);
	}

	#[test]
	fn act_again_when_re_inserted() {
		let mut app = setup();

		let mut entity = app.world_mut().spawn(GltfMeshName("name".to_owned()));
		entity.insert(GltfMeshName("other name".to_owned()));

		assert_eq!(
			Some(&AssetMeshName(NormalizedName::from(
				"other name".to_owned()
			))),
			entity.get::<AssetMeshName>(),
		);
	}
}
