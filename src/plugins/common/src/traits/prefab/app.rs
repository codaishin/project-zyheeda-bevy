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
	use super::*;
	use crate::{
		errors::{ErrorData, Level},
		traits::{load_asset::mock::MockAssetServer, prefab::PrefabEntityCommands},
	};
	use std::fmt::Display;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Asset;

	#[derive(Component)]
	struct _Component {
		prefab: Result<_Prefab<&'static str>, _Error>,
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Prefab<TAsset>(TAsset);

	struct _Dependency;

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
		fn level(&self) -> Level {
			Level::Error
		}

		fn label() -> impl Display {
			"_ERROR"
		}

		fn into_details(self) -> impl Display {
			self
		}
	}

	fn save_result(In(result): In<Result<(), _Error>>, mut commands: Commands) {
		commands.insert_resource(_Result(result));
	}

	fn setup(asset_server: MockAssetServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(asset_server);
		app.add_observer(
			instantiate_prefab::<_Component, _Dependency, MockAssetServer>.pipe(save_result),
		);

		app
	}

	#[test]
	fn call_prefab_instantiation_method() {
		let handle = new_handle();
		let mut app = setup(
			MockAssetServer::default()
				.path("my/path")
				.returns(handle.clone()),
		);

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
		let mut app = setup(MockAssetServer::default());

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
		let mut app = setup(MockAssetServer::default());

		app.world_mut()
			.spawn(_Component {
				prefab: Ok(_Prefab("my/path/a")),
			})
			.insert(_Component {
				prefab: Ok(_Prefab("my/path/b")),
			});

		let server = app.world().resource::<MockAssetServer>();
		assert_eq!(
			(1, 0),
			(server.calls("my/path/a"), server.calls("my/path/b")),
		);
	}
}
