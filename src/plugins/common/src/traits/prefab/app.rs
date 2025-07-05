use super::{AddPrefabObserver, Prefab};
use crate::{errors::Error, systems::log::OnError, traits::load_asset::LoadAsset};
use bevy::prelude::*;

impl AddPrefabObserver for App {
	fn add_prefab_observer<TPrefab, TDependencies>(&mut self) -> &mut Self
	where
		TPrefab: Prefab<TDependencies>,
		TDependencies: 'static,
	{
		self.add_observer(
			instantiate_prefab::<TPrefab, TDependencies, AssetServer>.pipe(OnError::log),
		)
	}
}

fn instantiate_prefab<TPrefab, TDependencies, TAssetServer>(
	trigger: Trigger<OnAdd, TPrefab>,
	components: Query<&TPrefab>,
	mut commands: Commands,
	mut asset_server: ResMut<TAssetServer>,
) -> Result<(), Error>
where
	TPrefab: Prefab<TDependencies>,
	TAssetServer: Resource + LoadAsset,
{
	let entity = trigger.target();
	let Ok(component) = components.get(entity) else {
		return Ok(());
	};
	let Ok(mut entity) = commands.get_entity(entity) else {
		return Ok(());
	};

	component.insert_prefab_components(&mut entity, asset_server.as_mut())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		errors::{Error, Level},
		traits::prefab::PrefabEntityCommands,
	};
	use bevy::asset::AssetPath;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Asset;

	#[derive(Component)]
	struct _Component {
		prefab: Result<_Prefab<&'static str>, Error>,
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Prefab<TAsset>(TAsset);

	struct _Dependency;

	#[derive(Resource, NestedMocks)]
	struct _AssetServer {
		mock: Mock_AssetServer,
	}

	#[automock]
	impl LoadAsset for _AssetServer {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	impl Prefab<_Dependency> for _Component {
		fn insert_prefab_components(
			&self,
			entity: &mut impl PrefabEntityCommands,
			asset_server: &mut impl LoadAsset,
		) -> Result<(), Error> {
			match &self.prefab {
				Ok(_Prefab(path)) => entity
					.try_insert_if_new(_Prefab::<Handle<_Asset>>(asset_server.load_asset(*path))),
				Err(error) => return Err(error.clone()),
			};

			Ok(())
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), Error>);

	fn save_result(In(result): In<Result<(), Error>>, mut commands: Commands) {
		commands.insert_resource(_Result(result));
	}

	fn setup(asset_server: _AssetServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(asset_server);
		app.add_observer(
			instantiate_prefab::<_Component, _Dependency, _AssetServer>.pipe(save_result),
		);

		app
	}

	#[test]
	fn call_prefab_instantiation_method() {
		let handle = new_handle();
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset::<_Asset, &str>()
				.times(1)
				.with(eq("my/path"))
				.return_const(handle.clone());
		}));

		let entity = app.world_mut().spawn(_Component {
			prefab: Ok(_Prefab("my/path")),
		});

		assert_eq!(
			Some(&_Prefab(handle)),
			entity.get::<_Prefab::<Handle<_Asset>>>()
		);
	}

	#[test]
	fn return_error() {
		let mut app = setup(
			_AssetServer::new().with_mock(|mock: &mut Mock_AssetServer| {
				mock.expect_load_asset::<_Asset, &str>()
					.return_const(new_handle());
			}),
		);
		let error = Error {
			msg: "my error".to_owned(),
			lvl: Level::Error,
		};

		let entity = app.world_mut().spawn(_Component {
			prefab: Err(error.clone()),
		});

		assert_eq!(Some(&_Result(Err(error))), entity.get_resource::<_Result>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(
			_AssetServer::new().with_mock(|mock: &mut Mock_AssetServer| {
				mock.expect_load_asset::<_Asset, &str>()
					.times(1)
					.return_const(new_handle());
			}),
		);

		let mut entity = app.world_mut().spawn(_Component {
			prefab: Ok(_Prefab("my/path/a")),
		});
		entity.insert(_Component {
			prefab: Ok(_Prefab("my/path/b")),
		});
	}
}
