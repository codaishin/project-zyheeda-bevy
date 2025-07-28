use crate::{
	context::{EntityLoadBuffer, LoadBuffer, SaveContext},
	errors::{DeserializationOrLockError, IOErrors, InsertionError, LockPoisonedError},
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
		move |commands, asset_server| {
			let Ok(mut context) = context.lock() else {
				return Err(DeserializationOrLockError::LockPoisoned(LockPoisonedError));
			};
			let entities = new_entities(commands, &mut context.buffers.load, asset_server)
				.with_components(&context.handlers.high_priority)
				.with_components(&context.handlers.low_priority);

			match entities.errors.0.as_slice() {
				[] => Ok(()),
				_ => Err(DeserializationOrLockError::DeserializationErrors(
					entities.errors,
				)),
			}
		}
	}
}

fn new_entities<'a, TAssetServer>(
	mut commands: Commands<'a, 'a>,
	buffer: &mut LoadBuffer,
	asset_server: ResMut<'a, TAssetServer>,
) -> EntitiesBuffer<'a, TAssetServer>
where
	TAssetServer: Resource,
{
	let entities = buffer
		.drain(..)
		.map(|buffer| (commands.spawn_empty().id(), buffer))
		.collect::<Vec<_>>();

	EntitiesBuffer {
		entities,
		commands,
		asset_server,
		errors: IOErrors(vec![]),
	}
}

struct EntitiesBuffer<'a, TAssetServer>
where
	TAssetServer: Resource,
{
	entities: Vec<(Entity, EntityLoadBuffer)>,
	commands: Commands<'a, 'a>,
	asset_server: ResMut<'a, TAssetServer>,
	errors: IOErrors<InsertionError>,
}

impl<'a, TAssetServer> EntitiesBuffer<'a, TAssetServer>
where
	TAssetServer: Resource + LoadAsset,
{
	fn with_components<TComponentHandler>(mut self, handlers: &[TComponentHandler]) -> Self
	where
		TComponentHandler: InsertEntityComponent<TAssetServer>,
	{
		let assets = &mut self.asset_server;

		for (entity, components) in self.entities.iter_mut() {
			let Ok(mut entity) = self.commands.get_entity(*entity) else {
				continue;
			};
			for handler in handlers {
				let Err(err) = handler.insert_component(&mut entity, components, assets) else {
					continue;
				};
				self.errors.0.push(InsertionError::Serde(err));
			}
		}

		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{context::EntityLoadBuffer, errors::SerdeJsonError, file_io::FileIO};
	use bevy::{
		asset::AssetPath,
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::traits::load_asset::LoadAsset;
	use serde::{Deserialize, Serialize};
	use std::{any::type_name, collections::HashMap, path::PathBuf, sync::LazyLock};
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
	struct _A(EntityLoadBuffer);

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _CountA {
		a_count: usize,
	}

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
		CountA,
		Error,
	}

	impl InsertEntityComponent<_LoadAsset> for _FakeHandler {
		fn insert_component<'a>(
			&self,
			entity: &mut EntityCommands<'a>,
			components: &mut EntityLoadBuffer,
			_: &mut _LoadAsset,
		) -> Result<(), SerdeJsonError> {
			match self {
				_FakeHandler::A => entity.insert(_A(components.clone())),
				_FakeHandler::B => entity.insert(_B(components.clone())),
				_FakeHandler::CountA => entity.insert(_CountA { a_count: 0 }),
				_FakeHandler::Error => {
					return Err(SerdeJsonError(serde::de::Error::custom(
						"Fool! I refuse deserialization",
					)));
				}
			};
			Ok(())
		}
	}

	static FILE_IO: LazyLock<FileIO> = LazyLock::new(|| FileIO::with_file(PathBuf::new()));

	macro_rules! non_observers {
		($app:expr) => {
			$app.world()
				.iter_entities()
				.filter(|e| !e.contains::<Observer>())
		};
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(
			|trigger: Trigger<OnInsert, _CountA>,
			 mut q: Query<&mut _CountA>,
			 a: Query<(), With<_A>>| {
				let Ok(mut knows) = q.get_mut(trigger.target()) else {
					panic!("THERE WAS NO `_KnowA` present");
				};

				knows.a_count = a.iter().count();
			},
		);
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
				.with_low_priority_handlers([_FakeHandler::A, _FakeHandler::B]),
		));

		_ = app
			.world_mut()
			.run_system_once(SaveContext::read_buffer_system(context))?;

		let [entity] = assert_count!(1, non_observers!(&app));
		assert_eq!(
			(Some(&_A(components.clone())), Some(&_B(components.clone()))),
			(entity.get::<_A>(), entity.get::<_B>())
		);
		Ok(())
	}

	#[test]
	fn spawn_entity_with_priority_component() -> Result<(), RunSystemError> {
		let mut app = setup();
		let components = HashMap::from([(
			type_name::<_A>().to_owned(),
			serde_json::from_str("null").unwrap(),
		)]);
		let context = Arc::new(Mutex::new(
			SaveContext::from(FILE_IO.clone())
				.with_load_buffer([components.clone()])
				.with_high_priority_handlers([_FakeHandler::A, _FakeHandler::B]),
		));

		_ = app
			.world_mut()
			.run_system_once(SaveContext::read_buffer_system(context))?;

		let [entity] = assert_count!(1, non_observers!(&app));
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
				.with_low_priority_handlers([_FakeHandler::A]),
		));

		_ = app
			.world_mut()
			.run_system_once(SaveContext::read_buffer_system(context))?;

		let [one, two] = assert_count!(2, non_observers!(&app));
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
	fn insert_priority_components_on_all_entities_first() -> Result<(), RunSystemError> {
		let mut app = setup();
		let components_for_entity_1 = HashMap::from([]);
		let components_for_entity_2 = HashMap::from([]);
		let context = Arc::new(Mutex::new(
			SaveContext::from(FILE_IO.clone())
				.with_load_buffer([components_for_entity_1, components_for_entity_2])
				.with_high_priority_handlers([_FakeHandler::A])
				.with_low_priority_handlers([_FakeHandler::CountA]),
		));

		_ = app
			.world_mut()
			.run_system_once(SaveContext::read_buffer_system(context))?;

		let [fst, snd] = assert_count!(2, non_observers!(&app));
		assert_eq!(
			(Some(&_CountA { a_count: 2 }), Some(&_CountA { a_count: 2 })),
			(fst.get::<_CountA>(), snd.get::<_CountA>())
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
				.with_high_priority_handlers([_FakeHandler::Error, _FakeHandler::Error])
				.with_low_priority_handlers([_FakeHandler::Error, _FakeHandler::Error]),
		));

		let result = app
			.world_mut()
			.run_system_once(SaveContext::read_buffer_system(context))?;

		assert_eq!(
			Err(DeserializationOrLockError::DeserializationErrors(IOErrors(
				vec![
					InsertionError::Serde(SerdeJsonError(serde::de::Error::custom(
						"Fool! I refuse deserialization"
					))),
					InsertionError::Serde(SerdeJsonError(serde::de::Error::custom(
						"Fool! I refuse deserialization"
					))),
					InsertionError::Serde(SerdeJsonError(serde::de::Error::custom(
						"Fool! I refuse deserialization"
					))),
					InsertionError::Serde(SerdeJsonError(serde::de::Error::custom(
						"Fool! I refuse deserialization"
					))),
				]
			))),
			result,
		);
		Ok(())
	}

	#[test]
	fn context_is_empty_when_ran() -> Result<(), RunSystemError> {
		let mut app = setup();
		let components = HashMap::from([(
			type_name::<_A>().to_owned(),
			serde_json::from_str("null").unwrap(),
		)]);
		let context = Arc::new(Mutex::new(
			SaveContext::from(FILE_IO.clone())
				.with_load_buffer([components.clone(), components.clone()])
				.with_low_priority_handlers([_FakeHandler::A, _FakeHandler::B]),
		));

		_ = app
			.world_mut()
			.run_system_once(SaveContext::read_buffer_system(context.clone()))?;

		let Ok(context) = context.lock() else {
			panic!("LOCK FAILED");
		};
		assert!(context.buffers.load.is_empty());
		Ok(())
	}
}
