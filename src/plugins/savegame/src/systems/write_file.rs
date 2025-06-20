use crate::{
	context::{ComponentString, SaveContext},
	errors::{ContextIOError, LockPoisonedError, SerdeJsonErrors},
	traits::write_file::WriteFile,
};
use bevy::prelude::*;
use std::{
	collections::HashSet,
	sync::{Arc, Mutex},
};

impl<TFileIO> SaveContext<TFileIO> {
	pub(crate) fn write_file_system(
		context: Arc<Mutex<Self>>,
	) -> impl Fn() -> Result<(), ContextIOError<TFileIO::TError>>
	where
		TFileIO: WriteFile,
	{
		move || {
			let mut context = match context.lock() {
				Err(_) => return Err(ContextIOError::LockPoisoned(LockPoisonedError)),
				Ok(context) => context,
			};

			context.write_and_flush()
		}
	}

	fn write_and_flush(&mut self) -> Result<(), ContextIOError<TFileIO::TError>>
	where
		TFileIO: WriteFile,
	{
		let mut errors = vec![];
		let entities = self
			.save_buffer
			.drain()
			.map(join_entity_components)
			.filter_map(|result| match result {
				Ok(value) => Some(value),
				Err(SerdeJsonErrors(json_errors)) => {
					errors.extend(json_errors);
					None
				}
			})
			.collect::<Vec<_>>()
			.join(",");

		if !errors.is_empty() {
			return Err(ContextIOError::SerdeErrors(SerdeJsonErrors(errors)));
		}

		self.io
			.write(&format!("[{entities}]"))
			.map_err(ContextIOError::FileError)
	}
}

fn join_entity_components(
	(_, component_strings): (Entity, HashSet<ComponentString>),
) -> Result<String, SerdeJsonErrors> {
	let mut errors = vec![];
	let components = component_strings
		.iter()
		.map(serde_json::to_string)
		.filter_map(|result| match result {
			Ok(value) => Some(value),
			Err(error) => {
				errors.push(error);
				None
			}
		})
		.collect::<Vec<_>>()
		.join(",");

	if !errors.is_empty() {
		return Err(SerdeJsonErrors(errors));
	}

	Ok(format!("[{components}]"))
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{simple_init, test_tools::utils::SingleThreadedApp, traits::mock::Mock};
	use mockall::{mock, predicate::eq};

	#[derive(Debug, PartialEq, Clone)]
	struct _Error;

	mock! {
	  _Writer {}
		impl WriteFile for _Writer {
			type TError = _Error;
			fn write(&self, string: &str) -> Result<(), _Error>;
		}
	}

	simple_init!(Mock_Writer);

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn call_write() -> Result<(), RunSystemError> {
		let string = ComponentString {
			comp: "A".to_owned(),
			dto: None,
			value: serde_json::from_str(r#"{"value": 32}"#).unwrap(),
		};
		let writer = Mock_Writer::new_mock(|mock| {
			mock.expect_write()
				.times(1)
				.with(eq(format!(
					"[[{}]]",
					serde_json::to_string(&string).unwrap()
				)))
				.return_const(Ok(()));
		});
		let context = Arc::new(Mutex::new(
			SaveContext::from(writer)
				.with_save_buffer([(Entity::from_raw(42), HashSet::from([string.clone()]))]),
		));
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(SaveContext::write_file_system(context))?;
		Ok(())
	}

	#[test]
	fn write_multiple_components_per_entity_on_flush() -> Result<(), RunSystemError> {
		let string_a = ComponentString {
			comp: "A".to_owned(),
			dto: None,
			value: serde_json::from_str(r#"{"value": 32}"#).unwrap(),
		};
		let string_b = ComponentString {
			comp: "B".to_owned(),
			dto: None,
			value: serde_json::from_str(r#"{"v": 42}"#).unwrap(),
		};
		let writer = Mock_Writer::new_mock(|mock| {
			mock.expect_write()
				.times(1)
				.withf(|v| {
					let a_b = format!(
						"[[{},{}]]",
						serde_json::to_string(&ComponentString {
							comp: "A".to_owned(),
							dto: None,
							value: serde_json::from_str(r#"{"value": 32}"#).unwrap(),
						})
						.unwrap(),
						serde_json::to_string(&ComponentString {
							comp: "B".to_owned(),
							dto: None,
							value: serde_json::from_str(r#"{"v": 42}"#).unwrap(),
						})
						.unwrap(),
					);
					let b_a = format!(
						"[[{},{}]]",
						serde_json::to_string(&ComponentString {
							comp: "B".to_owned(),
							dto: None,
							value: serde_json::from_str(r#"{"v": 42}"#).unwrap(),
						})
						.unwrap(),
						serde_json::to_string(&ComponentString {
							comp: "A".to_owned(),
							dto: None,
							value: serde_json::from_str(r#"{"value": 32}"#).unwrap(),
						})
						.unwrap(),
					);
					v == a_b || v == b_a
				})
				.return_const(Ok(()));
		});
		let context = Arc::new(Mutex::new(SaveContext::from(writer).with_save_buffer([(
			Entity::from_raw(42),
			HashSet::from([string_a.clone(), string_b.clone()]),
		)])));
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(SaveContext::write_file_system(context))?;
		Ok(())
	}

	#[test]
	fn write_multiple_entities() -> Result<(), RunSystemError> {
		let string_a = ComponentString {
			comp: "A".to_owned(),
			dto: None,
			value: serde_json::from_str(r#"{"value": 32}"#).unwrap(),
		};
		let string_b = ComponentString {
			comp: "B".to_owned(),
			dto: None,
			value: serde_json::from_str(r#"{"v": 42}"#).unwrap(),
		};
		let writer = Mock_Writer::new_mock(|mock| {
			mock.expect_write()
				.times(1)
				.withf(|v| {
					let a_b = format!(
						"[[{}],[{}]]",
						serde_json::to_string(&ComponentString {
							comp: "A".to_owned(),
							dto: None,
							value: serde_json::from_str(r#"{"value": 32}"#).unwrap(),
						})
						.unwrap(),
						serde_json::to_string(&ComponentString {
							comp: "B".to_owned(),
							dto: None,
							value: serde_json::from_str(r#"{"v": 42}"#).unwrap(),
						})
						.unwrap(),
					);
					let b_a = format!(
						"[[{}],[{}]]",
						serde_json::to_string(&ComponentString {
							comp: "B".to_owned(),
							dto: None,
							value: serde_json::from_str(r#"{"v": 42}"#).unwrap(),
						})
						.unwrap(),
						serde_json::to_string(&ComponentString {
							comp: "A".to_owned(),
							dto: None,
							value: serde_json::from_str(r#"{"value": 32}"#).unwrap(),
						})
						.unwrap(),
					);
					v == a_b || v == b_a
				})
				.return_const(Ok(()));
		});
		let context = Arc::new(Mutex::new(SaveContext::from(writer).with_save_buffer([
			(Entity::from_raw(42), HashSet::from([string_a.clone()])),
			(Entity::from_raw(43), HashSet::from([string_b.clone()])),
		])));
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(SaveContext::write_file_system(context))?;
		Ok(())
	}

	#[test]
	fn clear_save_buffer() -> Result<(), RunSystemError> {
		let writer = Mock_Writer::new_mock(|mock| {
			mock.expect_write().return_const(Ok(()));
		});
		let context = Arc::new(Mutex::new(SaveContext::from(writer).with_save_buffer([(
			Entity::from_raw(42),
			HashSet::from([ComponentString {
				comp: "A".to_owned(),
				dto: None,
				value: serde_json::from_str(r#"{"value": 32}"#).unwrap(),
			}]),
		)])));
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(SaveContext::write_file_system(context.clone()))?;

		assert!(
			context
				.lock()
				.expect("COULD NOT LOCK CONTEXT")
				.save_buffer
				.is_empty()
		);
		Ok(())
	}
}
