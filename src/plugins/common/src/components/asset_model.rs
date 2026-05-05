use crate::{
	components::{
		gltf::{GltfLookup, GltfScene},
		insert_asset::InsertAsset,
		load_model::LoadModel,
	},
	errors::Unreachable,
	traits::{
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Component, Debug, Default, PartialEq, Clone)]
#[require(Transform, Visibility)]
#[component(immutable)]
pub enum AssetModel {
	#[default]
	None,
	Scene(Scene),
	Mesh(InsertAsset<Mesh>),
}

impl AssetModel {
	pub fn scene<T>(params: T) -> Self
	where
		T: Into<Scene>,
	{
		Self::Scene(params.into())
	}
}

impl<TAssetServer> Prefab<TAssetServer> for AssetModel
where
	TAssetServer: Resource + LoadAsset,
{
	type TError = Unreachable;

	type TSystemParam<'w, 's> = ResMut<'w, TAssetServer>;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		mut asset_server: StaticSystemParam<ResMut<TAssetServer>>,
	) -> Result<(), Self::TError> {
		match &self {
			Self::Scene(scene) if *scene.use_gltf => {
				let gltf = asset_server.load_asset(scene.asset_path.clone());
				entity.try_insert((GltfLookup(gltf), GltfScene(scene.id)));
			}
			Self::Scene(scene) => {
				let root = asset_server.load_asset(
					GltfAssetLabel::Scene(*scene.id).from_asset(scene.asset_path.clone()),
				);
				entity.try_insert(LoadModel::Scene(root));
			}
			Self::None => {
				entity.try_insert(LoadModel::Scene(Handle::default()));
			}
			Self::Mesh(insert_mesh) => {
				entity.try_insert(insert_mesh.clone());
			}
		};

		Ok(())
	}
}

#[derive(Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize)]
pub struct SceneId(pub usize);

impl Deref for SceneId {
	type Target = usize;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Whether or not to add [`GltfLookup`] to the entity
#[derive(Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize)]
pub struct UseGltfLookup(pub bool);

impl Deref for UseGltfLookup {
	type Target = bool;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct Scene {
	pub asset_path: String,
	pub id: SceneId,
	pub use_gltf: UseGltfLookup,
}

impl Scene {
	pub const DEFAULT_SCENE_ID: SceneId = SceneId(0);
	pub const DEFAULT_GLTF_USAGE: UseGltfLookup = UseGltfLookup(false);
}

impl From<String> for Scene {
	fn from(asset_path: String) -> Self {
		Self {
			asset_path,
			id: Self::DEFAULT_SCENE_ID,
			use_gltf: Self::DEFAULT_GLTF_USAGE,
		}
	}
}

impl From<&'_ String> for Scene {
	fn from(asset_path: &String) -> Self {
		Self {
			asset_path: asset_path.clone(),
			id: Self::DEFAULT_SCENE_ID,
			use_gltf: Self::DEFAULT_GLTF_USAGE,
		}
	}
}

impl From<&'_ str> for Scene {
	fn from(asset_path: &str) -> Self {
		Self {
			asset_path: asset_path.into(),
			id: Self::DEFAULT_SCENE_ID,
			use_gltf: Self::DEFAULT_GLTF_USAGE,
		}
	}
}

impl<T> From<(T, SceneId)> for Scene
where
	T: Into<String>,
{
	fn from((asset_path, id): (T, SceneId)) -> Self {
		Self {
			asset_path: asset_path.into(),
			id,
			use_gltf: Self::DEFAULT_GLTF_USAGE,
		}
	}
}

impl<T> From<(T, SceneId, UseGltfLookup)> for Scene
where
	T: Into<String>,
{
	fn from((asset_path, id, use_gltf): (T, SceneId, UseGltfLookup)) -> Self {
		Self {
			asset_path: asset_path.into(),
			id,
			use_gltf,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::{load_asset::mock::MockAssetServer, prefab::AddPrefabObserver};
	use test_case::test_case;
	use testing::{SingleThreadedApp, new_handle};

	fn setup(asset_server: MockAssetServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(asset_server);
		app.add_prefab_observer::<AssetModel, MockAssetServer>();

		app
	}

	#[test_case(SceneId(0); "0")]
	#[test_case(SceneId(11); "11")]
	fn load_asset_scene(id: SceneId) {
		let handle = new_handle();
		let asset_path = "my/model.glb";
		let mut app = setup(
			MockAssetServer::default()
				.path(GltfAssetLabel::Scene(*id).from_asset(asset_path))
				.returns(handle.clone()),
		);

		let model = app
			.world_mut()
			.spawn(AssetModel::scene((asset_path, id)))
			.id();

		assert_eq!(
			Some(&LoadModel::Scene(handle)),
			app.world().entity(model).get::<LoadModel>(),
		);
	}

	#[test_case(SceneId(0); "0")]
	#[test_case(SceneId(11); "11")]
	fn load_asset_gltf(id: SceneId) {
		let handle = new_handle();
		let asset_path = "my/model.glb";
		let mut app = setup(
			MockAssetServer::default()
				.path(asset_path)
				.returns(handle.clone()),
		);

		let model = app
			.world_mut()
			.spawn(AssetModel::scene((asset_path, id, UseGltfLookup(true))))
			.id();

		assert_eq!(
			(Some(&GltfLookup(handle)), Some(&GltfScene(id))),
			(
				app.world().entity(model).get::<GltfLookup>(),
				app.world().entity(model).get::<GltfScene>(),
			)
		);
	}

	#[test]
	fn load_default_asset_when_set_to_none() {
		let mut app = setup(MockAssetServer::default());

		let model = app.world_mut().spawn(AssetModel::None).id();

		assert_eq!(
			Some(&LoadModel::Scene(Handle::default())),
			app.world().entity(model).get::<LoadModel>(),
		);
	}

	#[test]
	fn insert_procedural() {
		let mut app = setup(MockAssetServer::default());
		let insert_mesh = InsertAsset::unique(|| Sphere::new(3.).into());

		let model = app
			.world_mut()
			.spawn(AssetModel::Mesh(insert_mesh.clone()))
			.id();

		assert_eq!(
			Some(&insert_mesh),
			app.world().entity(model).get::<InsertAsset<Mesh>>(),
		);
	}
}
