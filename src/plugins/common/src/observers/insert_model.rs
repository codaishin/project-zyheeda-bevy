use crate::{
	components::model::Model,
	traits::accessors::get::GetMut,
	zyheeda_commands::ZyheedaCommands,
};
use bevy::prelude::*;

impl Model {
	pub(crate) fn insert(
		on_insert: On<Insert, Self>,
		mut commands: ZyheedaCommands,
		models: Query<&Self>,
	) {
		let Some(mut entity) = commands.get_mut(&on_insert.entity) else {
			return;
		};

		match models.get(entity.id()) {
			Ok(Self::Asset(m)) => {
				entity.try_insert(m.clone());
			}
			Ok(Self::Procedural(m)) => {
				entity.try_insert(m.clone());
			}
			Err(_) => {}
		}

		entity.try_remove::<Self>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{asset_model::AssetModel, insert_asset::InsertAsset};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(Model::insert);

		app
	}

	#[test]
	fn insert_asset() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(Model::Asset(AssetModel::from("asset/path")));

		assert_eq!(
			Some(&AssetModel::from("asset/path")),
			entity.get::<AssetModel>()
		);
	}

	#[test]
	fn insert_mesh() {
		fn sphere() -> Mesh {
			Mesh::from(Sphere::new(0.5))
		}

		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(Model::Procedural(InsertAsset::unique(sphere)));

		assert_eq!(
			Some(&InsertAsset::unique(sphere)),
			entity.get::<InsertAsset<Mesh>>()
		);
	}

	#[test]
	fn remove_model_component() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(Model::Asset(AssetModel::from("asset/path")));

		assert_eq!(None, entity.get::<Model>());
	}
}
