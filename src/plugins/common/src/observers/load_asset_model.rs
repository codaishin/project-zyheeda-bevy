use crate::{components::AssetModel, traits::load_asset::LoadAsset};
use bevy::prelude::*;

impl AssetModel {
	pub(crate) fn load(
		trigger: Trigger<OnAdd, AssetModel>,
		commands: Commands,
		asset_models: Query<&AssetModel>,
		asset_server: ResMut<AssetServer>,
	) {
		load_asset_model(trigger, commands, asset_models, asset_server);
	}
}

fn load_asset_model<TServer>(
	trigger: Trigger<OnAdd, AssetModel>,
	mut commands: Commands,
	asset_models: Query<&AssetModel>,
	mut asset_server: ResMut<TServer>,
) where
	for<'a> TServer: Resource + LoadAsset,
{
	let entity = trigger.target();

	let Ok(asset_model) = asset_models.get(entity) else {
		return;
	};

	let Ok(mut entity) = commands.get_entity(entity) else {
		return;
	};

	let handle = match asset_model {
		AssetModel::None => Handle::<Scene>::default(),
		AssetModel::Path(path) => {
			asset_server.load_asset(GltfAssetLabel::Scene(0).from_asset(path.clone()))
		}
	};

	entity.insert(SceneRoot(handle));
	entity.remove::<AssetModel>();
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::AssetModel, test_tools::utils::new_handle};
	use bevy::asset::AssetPath;
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Resource, NestedMocks)]
	struct _AssetServer {
		mock: Mock_AssetServer,
	}

	#[automock]
	impl LoadAsset for _AssetServer {
		fn load_asset<TAsset: Asset, TPath: Into<AssetPath<'static>> + 'static>(
			&mut self,
			path: TPath,
		) -> Handle<TAsset> {
			self.mock.load_asset(path)
		}
	}

	fn setup(asset_server: _AssetServer) -> App {
		let mut app = App::new();

		app.insert_resource(asset_server);
		app.add_observer(load_asset_model::<_AssetServer>);

		app
	}

	#[test]
	fn load_asset() {
		let handle = new_handle();
		let mut app = setup(
			_AssetServer::new().with_mock(|mock: &mut Mock_AssetServer| {
				mock.expect_load_asset::<Scene, AssetPath<'static>>()
					.return_const(handle.clone());
			}),
		);

		let model = app.world_mut().spawn(AssetModel::path("my/model.glb")).id();

		assert_eq!(
			Some(&SceneRoot(handle)),
			app.world().entity(model).get::<SceneRoot>(),
		);
	}

	#[test]
	fn load_default_asset_when_set_to_none() {
		let mut app = setup(
			_AssetServer::new().with_mock(|mock: &mut Mock_AssetServer| {
				mock.expect_load_asset::<Scene, AssetPath<'static>>()
					.return_const(new_handle());
			}),
		);

		let model = app.world_mut().spawn(AssetModel::None).id();

		assert_eq!(
			Some(&SceneRoot(Handle::default())),
			app.world().entity(model).get::<SceneRoot>(),
		);
	}

	#[test]
	fn load_asset_with_correct_path() {
		let mut app = setup(_AssetServer::new().with_mock(assert_correct_path));

		app.world_mut().spawn(AssetModel::path("my/model.glb"));

		fn assert_correct_path(mock: &mut Mock_AssetServer) {
			mock.expect_load_asset::<Scene, AssetPath<'static>>()
				.times(1)
				.with(eq(GltfAssetLabel::Scene(0).from_asset("my/model.glb")))
				.return_const(new_handle());
		}
	}

	#[test]
	fn remove_asset_model_component() {
		let mut app = setup(
			_AssetServer::new().with_mock(|mock: &mut Mock_AssetServer| {
				mock.expect_load_asset::<Scene, AssetPath<'static>>()
					.return_const(new_handle());
			}),
		);

		let model = app.world_mut().spawn(AssetModel::path("my/model.glb")).id();

		assert_eq!(None, app.world().entity(model).get::<AssetModel>(),);
	}
}
