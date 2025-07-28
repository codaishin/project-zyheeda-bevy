pub(crate) mod handler;

use crate::{
	context::handler::ComponentHandler,
	errors::{EntitySerializationErrors, SerdeJsonError},
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
	pub(crate) handlers: Handlers<TComponentHandler>,
	pub(crate) buffers: Buffers,
	pub(crate) io: TFileIO,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Handlers<TComponentHandler> {
	pub(crate) high_priority: Vec<TComponentHandler>,
	pub(crate) low_priority: Vec<TComponentHandler>,
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct Buffers {
	pub(crate) save: SaveBuffer,
	pub(crate) load: LoadBuffer,
}

#[cfg(test)]
impl<TFileIO, TComponentHandler> SaveContext<TFileIO, TComponentHandler> {
	pub(crate) fn with_save_buffer<T>(mut self, buffer: T) -> Self
	where
		T: Into<SaveBuffer>,
	{
		self.buffers.save = buffer.into();
		self
	}

	pub(crate) fn with_load_buffer<T>(mut self, buffer: T) -> Self
	where
		T: Into<LoadBuffer>,
	{
		self.buffers.load = buffer.into();
		self
	}

	pub(crate) fn with_low_priority_handlers<T>(mut self, handlers: T) -> Self
	where
		T: Into<Vec<TComponentHandler>>,
	{
		self.handlers.low_priority = handlers.into();
		self
	}

	pub(crate) fn with_high_priority_handlers<T>(mut self, handlers: T) -> Self
	where
		T: Into<Vec<TComponentHandler>>,
	{
		self.handlers.high_priority = handlers.into();
		self
	}
}

impl<TFileIO, TComponentHandler> From<TFileIO> for SaveContext<TFileIO, TComponentHandler> {
	fn from(io: TFileIO) -> Self {
		Self {
			io,
			handlers: Handlers::default(),
			buffers: Buffers::default(),
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
			.all()
			.filter_map(|handler| {
				handler
					.buffer_component(&mut self.buffers.save, entity)
					.map_err(SerdeJsonError)
					.err()
			})
			.collect::<Vec<_>>();

		match errors.as_slice() {
			[] => Ok(()),
			_ => Err(EntitySerializationErrors(errors)),
		}
	}
}

impl<TComponentHandler> Handlers<TComponentHandler> {
	fn all(&self) -> impl Iterator<Item = &TComponentHandler> {
		self.high_priority.iter().chain(self.low_priority.iter())
	}
}

impl<TComponentHandler> Default for Handlers<TComponentHandler> {
	fn default() -> Self {
		Self {
			high_priority: vec![],
			low_priority: vec![],
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
	use mockall::mock;
	use serde_json::from_str;
	use std::path::PathBuf;
	use testing::{Mock, SingleThreadedApp, simple_init};

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

	mod low_priority {
		use super::*;

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
					.with_low_priority_handlers([Mock_Handler::new_mock(|mock| {
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
			let mut context = SaveContext::from(FileIO::with_file(PathBuf::new()))
				.with_low_priority_handlers([
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

	mod high_priority {
		use super::*;

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
					.with_high_priority_handlers([Mock_Handler::new_mock(|mock| {
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
			let mut context = SaveContext::from(FileIO::with_file(PathBuf::new()))
				.with_high_priority_handlers([
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
}
