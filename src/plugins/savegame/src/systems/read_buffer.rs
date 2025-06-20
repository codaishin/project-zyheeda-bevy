use crate::{
	context::SaveContext,
	errors::{DeserializationOrLockError, LockPoisonedError, SerdeJsonErrors},
	file_io::FileIO,
	traits::insert_entity_component::InsertEntityComponent,
};
use bevy::prelude::*;
use common::traits::load_asset::LoadAsset;
use std::sync::{Arc, Mutex};

impl<T> SaveContext<FileIO, T> {
	pub(crate) fn read_buffer_system<TLoadAsset>(
		context: Arc<Mutex<Self>>,
	) -> impl Fn(Commands, ResMut<TLoadAsset>) -> Result<(), DeserializationOrLockError>
	where
		TLoadAsset: Resource + LoadAsset,
		T: InsertEntityComponent<TLoadAsset>,
	{
		move |mut commands, mut asset_server| {
			let Ok(context) = context.lock() else {
				return Err(DeserializationOrLockError::LockPoisoned(LockPoisonedError));
			};
			let server = asset_server.as_mut();

			let mut errors = vec![];

			for components in context.load_buffer.clone().iter_mut() {
				let entity = &mut commands.spawn_empty();
				for handler in &context.handlers {
					let Err(err) = handler.insert_component(entity, components, server) else {
						continue;
					};
					errors.push(err);
				}
			}

			match errors.as_slice() {
				[] => Ok(()),
				_ => Err(DeserializationOrLockError::DeserializationErrors(
					SerdeJsonErrors(errors),
				)),
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{context::EntityLoadBuffer, errors::SerdeJsonErrors, file_io::FileIO};
	use bevy::{
		asset::AssetPath,
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::{
		assert_count,
		test_tools::utils::SingleThreadedApp,
		traits::load_asset::LoadAsset,
	};
	use serde::{Deserialize, Serialize};
	use std::{any::type_name, collections::HashMap, path::PathBuf, sync::LazyLock};

	#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
	struct _A(EntityLoadBuffer);

	#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
	struct _B(EntityLoadBuffer);

	#[derive(Resource, Default, Debug, PartialEq)]
	struct _LoadAsset;

	impl LoadAsset for _LoadAsset {
		fn load_asset<TAsset, TPath>(&mut self, _: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			panic!("SHOULD NOT BE CALLED");
		}
	}

	enum _FakeHandler {
		A,
		B,
		Error,
	}

	impl InsertEntityComponent<_LoadAsset> for _FakeHandler {
		fn insert_component<'a>(
			&self,
			entity: &mut EntityCommands<'a>,
			components: &mut EntityLoadBuffer,
			_: &mut _LoadAsset,
		) -> Result<(), serde_json::Error> {
			match self {
				_FakeHandler::A => entity.insert(_A(components.clone())),
				_FakeHandler::B => entity.insert(_B(components.clone())),
				_FakeHandler::Error => {
					return Err(serde::de::Error::custom("Fool! I refuse deserialization"));
				}
			};
			Ok(())
		}
	}

	static FILE_IO: LazyLock<FileIO> = LazyLock::new(|| FileIO::with_file(PathBuf::new()));

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_LoadAsset>();

		app
	}

	#[test]
	fn spawn_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let components = HashMap::from([(
			type_name::<_A>().to_owned(),
			serde_json::from_str("null").unwrap(),
		)]);
		let context = Arc::new(Mutex::new(
			SaveContext::from(FILE_IO.clone())
				.with_load_buffer([components.clone()])
				.with_handlers([_FakeHandler::A, _FakeHandler::B]),
		));

		_ = app
			.world_mut()
			.run_system_once(SaveContext::read_buffer_system(context))?;

		let [entity] = assert_count!(1, app.world().iter_entities());
		assert_eq!(
			(Some(&_A(components.clone())), Some(&_B(components.clone()))),
			(entity.get::<_A>(), entity.get::<_B>())
		);
		Ok(())
	}

	#[test]
	fn spawn_multiple_entities() -> Result<(), RunSystemError> {
		let mut app = setup();
		let components_for_entity_1 = HashMap::from([(
			type_name::<_A>().to_owned(),
			serde_json::from_str("[1]").unwrap(),
		)]);
		let components_for_entity_2 = HashMap::from([(
			type_name::<_A>().to_owned(),
			serde_json::from_str("[2]").unwrap(),
		)]);
		let context = Arc::new(Mutex::new(
			SaveContext::from(FILE_IO.clone())
				.with_load_buffer([
					components_for_entity_1.clone(),
					components_for_entity_2.clone(),
				])
				.with_handlers([_FakeHandler::A]),
		));

		_ = app
			.world_mut()
			.run_system_once(SaveContext::read_buffer_system(context))?;

		let [one, two] = assert_count!(2, app.world().iter_entities());
		assert_eq!(
			(
				Some(&_A(components_for_entity_1.clone())),
				Some(&_A(components_for_entity_2.clone())),
			),
			(one.get::<_A>(), two.get::<_A>())
		);
		Ok(())
	}

	#[test]
	fn return_errors() -> Result<(), RunSystemError> {
		let mut app = setup();
		let components = HashMap::from([(
			type_name::<_A>().to_owned(),
			serde_json::from_str("null").unwrap(),
		)]);
		let context = Arc::new(Mutex::new(
			SaveContext::from(FILE_IO.clone())
				.with_load_buffer([components.clone()])
				.with_handlers([_FakeHandler::Error, _FakeHandler::Error]),
		));

		let result = app
			.world_mut()
			.run_system_once(SaveContext::read_buffer_system(context))?;

		assert_eq!(
			Err(DeserializationOrLockError::DeserializationErrors(
				SerdeJsonErrors(vec![
					serde::de::Error::custom("Fool! I refuse deserialization"),
					serde::de::Error::custom("Fool! I refuse deserialization"),
				])
			)),
			result,
		);
		Ok(())
	}
}
