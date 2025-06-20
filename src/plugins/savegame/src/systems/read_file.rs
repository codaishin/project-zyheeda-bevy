use crate::{
	context::SaveContext,
	errors::{ContextIOError, LockPoisonedError, SerdeJsonErrors},
	traits::read_file::ReadFile,
};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

impl<TFileIO> SaveContext<TFileIO>
where
	TFileIO: ReadFile,
{
	pub(crate) fn read_file_system(
		context: Arc<Mutex<Self>>,
	) -> impl Fn() -> Result<(), ContextIOError<TFileIO::TError>> {
		move || {
			let mut context = match context.lock() {
				Err(_) => return Err(ContextIOError::LockPoisoned(LockPoisonedError)),
				Ok(context) => context,
			};
			let entities = match context.io.read() {
				Err(e) => return Err(ContextIOError::FileError(e)),
				Ok(entities) => entities,
			};
			context.load_buffer = match serde_json::from_str(&entities) {
				Err(e) => return Err(ContextIOError::SerdeErrors(SerdeJsonErrors(vec![e]))),
				Ok(buffer) => buffer,
			};

			Ok(())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::context::{ComponentString, LoadBuffer};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{simple_init, test_tools::utils::SingleThreadedApp, traits::mock::Mock};
	use mockall::mock;
	use std::collections::HashSet;

	#[derive(Debug, PartialEq, Clone)]
	struct _Error;

	mock! {
		_Reader {}
		impl ReadFile for _Reader {
			type TError = _Error;
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
			dto: None,
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
			dto: None,
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
			vec![HashSet::from([ComponentString {
				comp: "A".to_owned(),
				dto: None,
				value: serde_json::from_str(r#"{"value": 32}"#).unwrap(),
			}])],
			context.lock().expect("COULD NOT LOCK CONTEXT").load_buffer
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
		let Err(error) = serde_json::from_str::<LoadBuffer>(&entities) else {
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
