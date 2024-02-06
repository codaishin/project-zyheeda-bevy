use crate::{errors::Error, traits::prefab::CreatePrefab};
use bevy::{
	asset::{Asset, Assets},
	ecs::system::{Commands, ResMut, Resource},
	render::mesh::Mesh,
};

pub fn register<TFor: CreatePrefab<TPrefab, TMaterial>, TPrefab: Resource, TMaterial: Asset>(
	mut commands: Commands,
	materials: ResMut<Assets<TMaterial>>,
	meshes: ResMut<Assets<Mesh>>,
) -> Result<(), Error> {
	let prefab = TFor::create_prefab(materials, meshes)?;
	commands.insert_resource(prefab);
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		errors::{Error, Level},
		systems::log::tests::{fake_log_error_lazy, FakeErrorLog},
	};
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Assets, Handle},
		ecs::{
			entity::Entity,
			system::{IntoSystem, ResMut},
		},
		pbr::StandardMaterial,
		render::{
			color::Color,
			mesh::{shape::Cube, Mesh},
		},
		utils::default,
	};

	#[derive(Resource)]
	struct _Prefab {
		material: Handle<StandardMaterial>,
		mesh: Handle<Mesh>,
	}

	struct _For;

	impl CreatePrefab<_Prefab, StandardMaterial> for _For {
		fn create_prefab(
			mut materials: ResMut<Assets<StandardMaterial>>,
			mut meshes: ResMut<Assets<Mesh>>,
		) -> Result<_Prefab, Error> {
			Ok(_Prefab {
				material: materials.add(StandardMaterial {
					base_color: Color::RED,
					..default()
				}),
				mesh: meshes.add(Mesh::from(Cube { size: 5. })),
			})
		}
	}

	struct _FaultyFor;

	impl _FaultyFor {
		fn error() -> Error {
			Error {
				msg: "Some Error".to_owned(),
				lvl: Level::Error,
			}
		}
	}

	impl CreatePrefab<_Prefab, StandardMaterial> for _FaultyFor {
		fn create_prefab(
			_: ResMut<Assets<StandardMaterial>>,
			_: ResMut<Assets<Mesh>>,
		) -> Result<_Prefab, Error> {
			Err(_FaultyFor::error())
		}
	}

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

	fn setup<TSource: CreatePrefab<_Prefab, StandardMaterial> + 'static>() -> (App, Entity) {
		let mut app = App::new();
		let logger = app.world.spawn_empty().id();

		app.init_resource::<Assets<StandardMaterial>>();
		app.init_resource::<Assets<Mesh>>();
		app.add_systems(
			Update,
			register::<TSource, _Prefab, StandardMaterial>.pipe(fake_log_error_lazy(logger)),
		);

		(app, logger)
	}

	#[test]
	fn get_material() {
		let (mut app, ..) = setup::<_For>();

		app.update();

		let handle = &app.world.resource::<_Prefab>().material;
		let material = get_original_asset_from_resources(&handle.id(), &app);

		assert_eq!(Some(Color::RED), material.map(|m| m.base_color))
	}

	#[test]
	fn get_mesh() {
		let (mut app, ..) = setup::<_For>();

		app.update();

		let handle = &app.world.resource::<_Prefab>().mesh;
		let mesh = get_original_asset_from_resources(&handle.id(), &app);

		assert_eq!(
			Some(Mesh::from(Cube { size: 5. }).primitive_topology()),
			mesh.map(|m| m.primitive_topology())
		)
	}

	#[test]
	fn ger_error() {
		let (mut app, logger) = setup::<_FaultyFor>();

		app.update();

		let log = app.world.entity(logger).get::<FakeErrorLog>();

		assert_eq!(Some(&FakeErrorLog(_FaultyFor::error())), log);
	}
}
