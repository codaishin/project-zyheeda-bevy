use crate::{
	context::{ComponentString, SaveContext},
	errors::{ContextIOError, LockPoisonedError, SerdeJsonErrors},
	traits::read_file::ReadFile,
};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

type SerializedEntities = Vec<Vec<ComponentString>>;

impl<TFileIO> SaveContext<TFileIO>
where
	TFileIO: ReadFile,
{
	pub(crate) fn read_file_system(
		context: Arc<Mutex<Self>>,
	) -> impl Fn() -> Result<(), ContextIOError<TFileIO::TReadError>> {
		move || {
			let mut context = match context.lock() {
				Err(_) => return Err(ContextIOError::LockPoisoned(LockPoisonedError)),
				Ok(context) => context,
			};
			let entities = match context.io.read() {
				Err(e) => return Err(ContextIOError::FileError(e)),
				Ok(entities) => entities,
			};
			let entities = match serde_json::from_str::<SerializedEntities>(&entities) {
				Err(e) => return Err(ContextIOError::SerdeErrors(SerdeJsonErrors(vec![e]))),
				Ok(components) => components,
			};

			context.buffers.load = entities
				.into_iter()
				.map(|components| {
					components
						.into_iter()
						.map(|ComponentString { comp, value }| (comp, value))
						.collect()
				})
				.collect();

			Ok(())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::context::ComponentString;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use mockall::mock;
	use std::collections::HashMap;
	use testing::{Mock, SingleThreadedApp, simple_init};

	#[derive(Debug, PartialEq, Clone)]
	struct _Error;

	mock! {
		_Reader {}
		impl ReadFile for _Reader {
			type TReadError = _Error;
			fn read(&self) -> Result<String, _Error>;
		}
	}

	simple_init!(Mock_Reader);

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn call_read() -> Result<(), RunSystemError> {
		let component = serde_json::to_string(&ComponentString {
			comp: "A".to_owned(),
			value: serde_json::from_str(r#"{"value": 32}"#).unwrap(),
		})
		.unwrap();
		let entity = format!("[{component}]");
		let entities = format!("[{entity}]");
		let reader = Mock_Reader::new_mock(|mock| {
			mock.expect_read()
				.times(1)
				.return_const(Ok(entities.clone()));
		});
		let context = Arc::new(Mutex::new(SaveContext::from(reader)));
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(SaveContext::read_file_system(context))?;
		Ok(())
	}

	#[test]
	fn write_load_buffer() -> Result<(), RunSystemError> {
		let component = serde_json::to_string(&ComponentString {
			comp: "A".to_owned(),
			value: serde_json::from_str(r#"{"value": 32}"#).unwrap(),
		})
		.unwrap();
		let entity = format!("[{component}]");
		let entities = format!("[{entity}]");
		let reader = Mock_Reader::new_mock(|mock| {
			mock.expect_read().return_const(Ok(entities.clone()));
		});
		let context = Arc::new(Mutex::new(SaveContext::from(reader)));
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(SaveContext::read_file_system(context.clone()))?;

		assert_eq!(
			vec![HashMap::from([(
				"A".to_owned(),
				serde_json::from_str(r#"{"value": 32}"#).unwrap(),
			)])],
			context.lock().expect("COULD NOT LOCK CONTEXT").buffers.load
		);
		Ok(())
	}

	#[test]
	fn return_read_error() -> Result<(), RunSystemError> {
		let reader = Mock_Reader::new_mock(|mock| {
			mock.expect_read().return_const(Err(_Error));
		});
		let context = Arc::new(Mutex::new(SaveContext::from(reader)));
		let mut app = setup();

		let result = app
			.world_mut()
			.run_system_once(SaveContext::read_file_system(context.clone()))?;

		assert_eq!(Err(ContextIOError::FileError(_Error)), result);
		Ok(())
	}

	#[test]
	fn deserialize_error() -> Result<(), RunSystemError> {
		let entities = "[my so very broken entity]".to_owned();
		let Err(error) = serde_json::from_str::<SerializedEntities>(&entities) else {
			panic!("SETUP BROKEN: EXPECTED ERROR");
		};
		let reader = Mock_Reader::new_mock(|mock| {
			mock.expect_read()
				.times(1)
				.return_const(Ok(entities.clone()));
		});
		let context = Arc::new(Mutex::new(SaveContext::from(reader)));
		let mut app = setup();

		let result = app
			.world_mut()
			.run_system_once(SaveContext::read_file_system(context.clone()))?;

		assert_eq!(
			Err(ContextIOError::SerdeErrors(SerdeJsonErrors(vec![error]))),
			result
		);
		Ok(())
	}
}
