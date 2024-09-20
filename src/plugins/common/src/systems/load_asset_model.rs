use crate::{components::AssetModel, traits::load_asset::LoadAsset};
use bevy::{
	prelude::{Commands, Entity, Query, ResMut, Resource},
	scene::Scene,
};

pub(crate) fn load_asset_model<TServer: Resource + LoadAsset>(
	mut commands: Commands,
	asset_models: Query<(Entity, &AssetModel)>,
	mut asset_server: ResMut<TServer>,
) {
	for (entity, AssetModel(path)) in &asset_models {
		let Some(mut entity) = commands.get_entity(entity) else {
			continue;
		};

		let handle = asset_server.load_asset::<Scene>((*path).into());
		entity.insert(handle);
		entity.remove::<AssetModel>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::AssetModel, test_tools::utils::new_handle, traits::load_asset::Path};
	use bevy::{
		app::App,
		asset::{Asset, Handle},
		ecs::system::RunSystemOnce,
		prelude::Resource,
		scene::Scene,
	};
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Resource, NestedMocks)]
	struct _AssetServer {
		mock: Mock_AssetServer,
	}

	#[automock]
	impl LoadAsset for _AssetServer {
		fn load_asset<TAsset: Asset>(&mut self, path: Path) -> Handle<TAsset> {
			self.mock.load_asset(path)
		}
	}

	fn setup(asset_server: _AssetServer) -> App {
		let mut app = App::new();
		app.insert_resource(asset_server);

		app
	}

	#[test]
	fn load_asset() {
		let handle = new_handle();
		let mut app = setup(
			_AssetServer::new().with_mock(|mock: &mut Mock_AssetServer| {
				mock.expect_load_asset::<Scene>()
					.return_const(handle.clone());
			}),
		);
		let model = app
			.world_mut()
			.spawn(AssetModel("my/model.glb#Scene0"))
			.id();

		app.world_mut()
			.run_system_once(load_asset_model::<_AssetServer>);

		assert_eq!(
			Some(&handle),
			app.world().entity(model).get::<Handle<Scene>>(),
		);
	}

	#[test]
	fn load_asset_with_correct_path() {
		let mut app = setup(_AssetServer::new().with_mock(assert_correct_path));
		app.world_mut().spawn(AssetModel("my/model.glb#Scene0"));

		app.world_mut()
			.run_system_once(load_asset_model::<_AssetServer>);

		fn assert_correct_path(mock: &mut Mock_AssetServer) {
			mock.expect_load_asset::<Scene>()
				.times(1)
				.with(eq(Path::from("my/model.glb#Scene0")))
				.return_const(new_handle());
		}
	}

	#[test]
	fn remove_asset_model_component() {
		let mut app = setup(
			_AssetServer::new().with_mock(|mock: &mut Mock_AssetServer| {
				mock.expect_load_asset::<Scene>().return_const(new_handle());
			}),
		);
		let model = app
			.world_mut()
			.spawn(AssetModel("my/model.glb#Scene0"))
			.id();

		app.world_mut()
			.run_system_once(load_asset_model::<_AssetServer>);

		assert_eq!(None, app.world().entity(model).get::<AssetModel>(),);
	}
}
