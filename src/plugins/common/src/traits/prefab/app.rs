use super::{AddPrefabObserver, Prefab};
use crate::{systems::log::OnError, traits::load_asset::LoadAsset};
use bevy::prelude::*;

impl AddPrefabObserver for App {
	fn add_prefab_observer<TPrefab, TDependencies>(&mut self) -> &mut Self
	where
		TPrefab: Prefab<TDependencies> + Component,
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
) -> Result<(), TPrefab::TError>
where
	TPrefab: Prefab<TDependencies> + Component,
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
	use std::fmt::Display;

	use super::*;
	use crate::{
		errors::{ErrorData, Level},
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
		prefab: Result<_Prefab<&'static str>, _Error>,
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
		type TError = _Error;

		fn insert_prefab_components(
			&self,
			entity: &mut impl PrefabEntityCommands,
			asset_server: &mut impl LoadAsset,
		) -> Result<(), _Error> {
			match &self.prefab {
				Ok(_Prefab(path)) => entity
					.try_insert_if_new(_Prefab::<Handle<_Asset>>(asset_server.load_asset(*path))),
				Err(error) => return Err(*error),
			};

			Ok(())
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), _Error>);

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Error;

	impl Display for _Error {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "_ERROR")
		}
	}

	impl ErrorData for _Error {
		type TContext = Self;

		fn level(&self) -> Level {
			Level::Error
		}

		fn label() -> String {
			"_ERROR".to_owned()
		}

		fn context(&self) -> &Self::TContext {
			self
		}
	}

	fn save_result(In(result): In<Result<(), _Error>>, mut commands: Commands) {
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

		let entity = app.world_mut().spawn(_Component {
			prefab: Err(_Error),
		});

		assert_eq!(
			Some(&_Result(Err(_Error))),
			entity.get_resource::<_Result>(),
		);
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
