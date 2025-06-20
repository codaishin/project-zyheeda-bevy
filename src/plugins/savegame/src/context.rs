pub(crate) mod handler;

use crate::{
	context::handler::ComponentHandler,
	errors::EntitySerializationErrors,
	file_io::FileIO,
	traits::{buffer_entity_component::BufferEntityComponent, write_buffer::WriteBuffer},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub(crate) type SaveBuffer = HashMap<Entity, HashSet<ComponentString>>;
pub(crate) type EntityLoadBuffer = HashMap<String, Value>;
pub(crate) type LoadBuffer = Vec<EntityLoadBuffer>;

#[derive(Debug, PartialEq, Default)]
pub struct SaveContext<TFileIO = FileIO, TComponentHandler = ComponentHandler> {
	pub(crate) handlers: Vec<TComponentHandler>,
	pub(crate) save_buffer: SaveBuffer,
	pub(crate) load_buffer: LoadBuffer,
	pub(crate) io: TFileIO,
}

#[cfg(test)]
impl<TFileIO, TComponentHandler> SaveContext<TFileIO, TComponentHandler> {
	pub(crate) fn with_save_buffer<T>(mut self, buffer: T) -> Self
	where
		T: Into<SaveBuffer>,
	{
		self.save_buffer = buffer.into();
		self
	}

	pub(crate) fn with_load_buffer<T>(mut self, buffer: T) -> Self
	where
		T: Into<LoadBuffer>,
	{
		self.load_buffer = buffer.into();
		self
	}

	pub(crate) fn with_handlers<T>(mut self, handlers: T) -> Self
	where
		T: Into<Vec<TComponentHandler>>,
	{
		self.handlers = handlers.into();
		self
	}
}

impl<TFileIO, TComponentHandler> From<TFileIO> for SaveContext<TFileIO, TComponentHandler> {
	fn from(io: TFileIO) -> Self {
		Self {
			io,
			handlers: vec![],
			load_buffer: vec![],
			save_buffer: HashMap::default(),
		}
	}
}

impl<TFileIO, TComponentHandler> WriteBuffer for SaveContext<TFileIO, TComponentHandler>
where
	TComponentHandler: BufferEntityComponent,
{
	fn write_buffer(&mut self, entity: EntityRef) -> Result<(), EntitySerializationErrors> {
		let errors = self
			.handlers
			.iter()
			.filter_map(|handler| {
				handler
					.buffer_component(&mut self.save_buffer, entity)
					.err()
			})
			.collect::<Vec<_>>();

		match errors.as_slice() {
			[] => Ok(()),
			_ => Err(EntitySerializationErrors(errors)),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub(crate) struct ComponentString {
	/// Component type name, abbreviated to reduce memory usage
	pub(crate) comp: String,

	/// Component serialized value
	pub(crate) value: Value,
}

#[cfg(test)]
mod test_write_buffer {
	use super::*;
	use common::{simple_init, test_tools::utils::SingleThreadedApp, traits::mock::Mock};
	use mockall::mock;
	use serde_json::from_str;
	use std::path::PathBuf;

	mock! {
		_Handler {}
		impl BufferEntityComponent for _Handler {
			fn buffer_component<'a>(&self, buffer: &mut SaveBuffer, entity: EntityRef<'a>) -> Result<(), serde_json::Error>;
		}
	}

	simple_init!(Mock_Handler);

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn buffer() {
		fn get_buffer() -> SaveBuffer {
			HashMap::from([(
				Entity::from_raw(42),
				HashSet::from([ComponentString {
					comp: "name".to_owned(),
					value: from_str("[\"state\"]").unwrap(),
				}]),
			)])
		}

		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let entity = app.world().entity(entity);
		let id = entity.id();
		let mut context =
			SaveContext::<FileIO, Mock_Handler>::from(FileIO::with_file(PathBuf::new()))
				.with_save_buffer(get_buffer())
				.with_handlers([Mock_Handler::new_mock(|mock| {
					mock.expect_buffer_component()
						.times(1)
						.returning(move |buffer, entity| {
							assert_eq!((&mut get_buffer(), id), (buffer, entity.id()));
							Ok(())
						});
				})]);

		let result = context.write_buffer(entity);

		assert!(result.is_ok());
	}

	#[test]
	fn buffer_error() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let entity = app.world().entity(entity);
		let mut context = SaveContext::from(FileIO::with_file(PathBuf::new())).with_handlers([
			Mock_Handler::new_mock(|mock| {
				mock.expect_buffer_component()
					.returning(|_, _| Err(serde::ser::Error::custom("NOPE, U LOSE")));
			}),
			Mock_Handler::new_mock(|mock| {
				mock.expect_buffer_component()
					.returning(|_, _| Err(serde::ser::Error::custom("NOPE, U LOSE AGAIN")));
			}),
		]);

		let result = context.write_buffer(entity);

		assert_eq!(
			Err("NOPE, U LOSE|NOPE, U LOSE AGAIN".to_owned()),
			result.map_err(|EntitySerializationErrors(errors)| errors
				.iter()
				.map(|error| error.to_string())
				.collect::<Vec<_>>()
				.join("|"))
		);
	}
}
