use crate::{resources::ModelData, traits::model::Model};
use bevy::{
	asset::{Asset, Assets},
	ecs::system::{Commands, ResMut},
	render::mesh::Mesh,
};

pub fn store_model_data<
	TMaterial: Asset + Sync + Send + 'static,
	TModel: Model<TMaterial> + Sync + Send + 'static,
>(
	mut commands: Commands,
	mut materials: ResMut<Assets<TMaterial>>,
	mut meshes: ResMut<Assets<Mesh>>,
) {
	commands.insert_resource(ModelData::<TMaterial, TModel>::new(
		materials.add(TModel::material()),
		meshes.add(TModel::mesh()),
	));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::resources::ModelData;
	use bevy::{
		self,
		app::{App, Update},
		asset::AssetId,
		pbr::StandardMaterial,
		prelude::default,
		render::{color::Color, mesh::shape::Cube},
	};

	fn get_original_asset_from_resources<'a, TAsset: Asset>(
		seek: &AssetId<TAsset>,
		app: &'a App,
	) -> Option<&'a TAsset> {
		let assets = app.world.resource::<Assets<TAsset>>();
		let assets: Vec<_> = assets.iter().collect();
		assets
			.iter()
			.find_map(|(id, asset)| if id == seek { Some(asset) } else { None })
			.cloned()
	}

	struct _Model;

	impl Model<StandardMaterial> for _Model {
		fn material() -> StandardMaterial {
			StandardMaterial {
				base_color: Color::RED,
				..default()
			}
		}
		fn mesh() -> Mesh {
			Cube { size: 5. }.into()
		}
	}

	fn setup() -> App {
		let mut app = App::new();

		app.init_resource::<Assets<StandardMaterial>>();
		app.init_resource::<Assets<Mesh>>();
		app.add_systems(Update, store_model_data::<StandardMaterial, _Model>);

		app
	}

	#[test]
	fn get_material() {
		let mut app = setup();

		app.update();

		let handle = &app
			.world
			.resource::<ModelData<StandardMaterial, _Model>>()
			.material;
		let material = get_original_asset_from_resources(&handle.id(), &app);

		assert_eq!(
			Some(_Model::material().base_color),
			material.map(|m| m.base_color)
		)
	}

	#[test]
	fn get_mesh() {
		let mut app = setup();

		app.update();

		let handle = &app
			.world
			.resource::<ModelData<StandardMaterial, _Model>>()
			.mesh;
		let mesh = get_original_asset_from_resources(&handle.id(), &app);

		assert_eq!(
			Some(_Model::mesh().primitive_topology()),
			mesh.map(|m| m.primitive_topology())
		)
	}
}
