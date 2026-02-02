use crate::{
	components::asset_model::{AssetModel, Model},
	traits::{accessors::get::GetMut, load_asset::LoadAsset},
	zyheeda_commands::ZyheedaCommands,
};
use bevy::prelude::*;

impl AssetModel {
	pub(crate) fn load(
		trigger: On<Add, AssetModel>,
		commands: ZyheedaCommands,
		asset_models: Query<&AssetModel>,
		asset_server: ResMut<AssetServer>,
	) {
		load_asset_model(trigger, commands, asset_models, asset_server);
	}
}

fn load_asset_model<TServer>(
	trigger: On<Add, AssetModel>,
	mut commands: ZyheedaCommands,
	asset_models: Query<&AssetModel>,
	mut asset_server: ResMut<TServer>,
) where
	for<'a> TServer: Resource + LoadAsset,
{
	let entity = trigger.entity;

	let Ok(asset_model) = asset_models.get(entity) else {
		return;
	};

	let Some(mut entity) = commands.get_mut(&entity) else {
		return;
	};

	let handle = match &asset_model.model {
		Model::None => Handle::<Scene>::default(),
		Model::Path(path) => {
			asset_server.load_asset(GltfAssetLabel::Scene(0).from_asset(path.clone()))
		}
	};

	entity.try_insert(SceneRoot(handle));
	entity.try_remove::<AssetModel>();
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::load_asset::mock::MockAssetServer;
	use testing::new_handle;

	fn setup(asset_server: MockAssetServer) -> App {
		let mut app = App::new();

		app.insert_resource(asset_server);
		app.add_observer(load_asset_model::<MockAssetServer>);

		app
	}

	#[test]
	fn load_asset() {
		let handle = new_handle();
		let path = "my/model.glb";
		let mut app = setup(
			MockAssetServer::default()
				.path(GltfAssetLabel::Scene(0).from_asset(path))
				.returns(handle.clone()),
		);

		let model = app.world_mut().spawn(AssetModel::path(path)).id();

		assert_eq!(
			Some(&SceneRoot(handle)),
			app.world().entity(model).get::<SceneRoot>(),
		);
	}

	#[test]
	fn load_default_asset_when_set_to_none() {
		let mut app = setup(MockAssetServer::default());

		let model = app.world_mut().spawn(AssetModel::none()).id();

		assert_eq!(
			Some(&SceneRoot(Handle::default())),
			app.world().entity(model).get::<SceneRoot>(),
		);
	}

	#[test]
	fn remove_asset_model_component() {
		let mut app = setup(MockAssetServer::default());

		let model = app.world_mut().spawn(AssetModel::path("my/model.glb")).id();

		assert_eq!(None, app.world().entity(model).get::<AssetModel>(),);
	}
}
