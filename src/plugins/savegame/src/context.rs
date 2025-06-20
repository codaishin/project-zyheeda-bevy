use crate::{
	errors::{ContextFlushError, EntitySerializationErrors, LockPoisonedError, SerdeJsonErrors},
	traits::{write_buffer::WriteBuffer, write_file::WriteFile},
	writer::FileWriter,
};
use bevy::prelude::*;
use serde::Serialize;
use serde_json::{Error, Value, to_string, to_value};
use std::{
	any::type_name,
	collections::{HashMap, HashSet, hash_map::Entry},
	sync::{Arc, Mutex},
};

pub(crate) type Buffer = HashMap<Entity, HashSet<ComponentString>>;
pub(crate) type Handlers = Vec<fn(&mut Buffer, EntityRef) -> Result<(), Error>>;

#[derive(Debug, PartialEq, Default)]
pub struct SaveContext<TFileWriter = FileWriter> {
	pub(crate) handlers: Handlers,
	writer: TFileWriter,
	buffer: Buffer,
}

impl SaveContext {
	pub(crate) fn handle<T, TDto>(
		buffer: &mut HashMap<Entity, HashSet<ComponentString>>,
		entity: EntityRef,
	) -> Result<(), Error>
	where
		T: Component + Clone,
		TDto: From<T> + Serialize,
	{
		let Some(component) = entity.get::<T>() else {
			return Ok(());
		};
		let comp = type_name::<T>();
		let component_str = ComponentString {
			comp,
			dto: match type_name::<TDto>() {
				dto if dto == comp => None,
				dto => Some(dto),
			},
			value: to_value(TDto::from(component.clone()))?,
		};

		match buffer.entry(entity.id()) {
			Entry::Occupied(mut occupied_entry) => {
				occupied_entry.get_mut().insert(component_str);
			}
			Entry::Vacant(vacant_entry) => {
				vacant_entry.insert(HashSet::from([component_str]));
			}
		};
		Ok(())
	}
}

impl<TFileWriter> SaveContext<TFileWriter> {
	pub(crate) fn write_file_system(
		context: Arc<Mutex<Self>>,
	) -> impl Fn() -> Result<(), ContextFlushError<TFileWriter::TError>>
	where
		TFileWriter: WriteFile,
	{
		move || {
			let mut context = match context.lock() {
				Err(_) => return Err(ContextFlushError::LockPoisoned(LockPoisonedError)),
				Ok(context) => context,
			};

			context.write_and_flush()
		}
	}

	fn write_and_flush(&mut self) -> Result<(), ContextFlushError<TFileWriter::TError>>
	where
		TFileWriter: WriteFile,
	{
		let mut errors = vec![];
		let entities = self
			.buffer
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
			return Err(ContextFlushError::SerdeErrors(SerdeJsonErrors(errors)));
		}

		self.writer
			.write(&format!("[{entities}]"))
			.map_err(ContextFlushError::WriteError)
	}
}

impl<TFileWriter> From<TFileWriter> for SaveContext<TFileWriter> {
	fn from(writer: TFileWriter) -> Self {
		Self {
			writer,
			handlers: vec![],
			buffer: HashMap::default(),
		}
	}
}

fn join_entity_components(
	(_, component_strings): (Entity, HashSet<ComponentString>),
) -> Result<String, SerdeJsonErrors> {
	let mut errors = vec![];
	let components = component_strings
		.iter()
		.map(to_string)
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

impl WriteBuffer for SaveContext {
	fn write_buffer(&mut self, entity: EntityRef) -> Result<(), EntitySerializationErrors> {
		let errors = self
			.handlers
			.iter()
			.filter_map(|handler| handler(&mut self.buffer, entity).err())
			.collect::<Vec<_>>();

		match errors.as_slice() {
			[] => Ok(()),
			_ => Err(EntitySerializationErrors(errors)),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Clone)]
pub(crate) struct ComponentString {
	/// Component type name, abbreviated to reduce memory usage
	comp: &'static str,

	/// Component dto type name, abbreviated to reduce memory usage
	#[serde(skip_serializing_if = "Option::is_none")]
	dto: Option<&'static str>,

	/// Component serialized value
	value: Value,
}

#[cfg(test)]
mod test_flush {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{simple_init, test_tools::utils::SingleThreadedApp, traits::mock::Mock};
	use mockall::{mock, predicate::eq};
	use serde_json::{from_str, to_string};

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
	fn write_on_flush() -> Result<(), RunSystemError> {
		let string_a = ComponentString {
			comp: "A",
			dto: None,
			value: from_str(r#"{"value": 32}"#).unwrap(),
		};
		let context = Arc::new(Mutex::new(SaveContext {
			buffer: HashMap::from([(Entity::from_raw(11), HashSet::from([string_a.clone()]))]),
			writer: Mock_Writer::new_mock(|mock| {
				mock.expect_write()
					.times(1)
					.with(eq(format!("[[{}]]", to_string(&string_a).unwrap())))
					.return_const(Ok(()));
			}),
			handlers: vec![],
		}));
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(SaveContext::write_file_system(context))?;
		Ok(())
	}

	#[test]
	fn write_multiple_components_per_entity_on_flush() -> Result<(), RunSystemError> {
		let string_a = ComponentString {
			comp: "A",
			dto: None,
			value: from_str(r#"{"value": 32}"#).unwrap(),
		};
		let string_b = ComponentString {
			comp: "B",
			dto: None,
			value: from_str(r#"{"v": 42}"#).unwrap(),
		};
		let context = Arc::new(Mutex::new(SaveContext {
			buffer: HashMap::from([(
				Entity::from_raw(11),
				HashSet::from([string_a.clone(), string_b.clone()]),
			)]),
			writer: Mock_Writer::new_mock(|mock| {
				mock.expect_write()
					.times(1)
					.withf(|v| {
						let a_b = format!(
							"[[{},{}]]",
							to_string(&ComponentString {
								comp: "A",
								dto: None,
								value: from_str(r#"{"value": 32}"#).unwrap(),
							})
							.unwrap(),
							to_string(&ComponentString {
								comp: "B",
								dto: None,
								value: from_str(r#"{"v": 42}"#).unwrap(),
							})
							.unwrap(),
						);
						let b_a = format!(
							"[[{},{}]]",
							to_string(&ComponentString {
								comp: "B",
								dto: None,
								value: from_str(r#"{"v": 42}"#).unwrap(),
							})
							.unwrap(),
							to_string(&ComponentString {
								comp: "A",
								dto: None,
								value: from_str(r#"{"value": 32}"#).unwrap(),
							})
							.unwrap(),
						);
						v == a_b || v == b_a
					})
					.return_const(Ok(()));
			}),
			handlers: vec![],
		}));
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(SaveContext::write_file_system(context))?;
		Ok(())
	}

	#[test]
	fn write_multiple_entities_on_flush() -> Result<(), RunSystemError> {
		let string_a = ComponentString {
			comp: "A",
			dto: None,
			value: from_str(r#"{"value": 32}"#).unwrap(),
		};
		let string_b = ComponentString {
			comp: "B",
			dto: None,
			value: from_str(r#"{"v": 42}"#).unwrap(),
		};
		let context = Arc::new(Mutex::new(SaveContext {
			buffer: HashMap::from([
				(Entity::from_raw(11), HashSet::from([string_a.clone()])),
				(Entity::from_raw(12), HashSet::from([string_b.clone()])),
			]),
			writer: Mock_Writer::new_mock(|mock| {
				mock.expect_write()
					.times(1)
					.withf(|v| {
						let a_b = format!(
							"[[{}],[{}]]",
							to_string(&ComponentString {
								comp: "A",
								dto: None,
								value: from_str(r#"{"value": 32}"#).unwrap(),
							})
							.unwrap(),
							to_string(&ComponentString {
								comp: "B",
								dto: None,
								value: from_str(r#"{"v": 42}"#).unwrap(),
							})
							.unwrap(),
						);
						let b_a = format!(
							"[[{}],[{}]]",
							to_string(&ComponentString {
								comp: "B",
								dto: None,
								value: from_str(r#"{"v": 42}"#).unwrap(),
							})
							.unwrap(),
							to_string(&ComponentString {
								comp: "A",
								dto: None,
								value: from_str(r#"{"value": 32}"#).unwrap(),
							})
							.unwrap(),
						);
						v == a_b || v == b_a
					})
					.return_const(Ok(()));
			}),
			handlers: vec![],
		}));
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(SaveContext::write_file_system(context))?;
		Ok(())
	}

	#[test]
	fn clear_buffer_on_flush() -> Result<(), RunSystemError> {
		let context = Arc::new(Mutex::new(SaveContext {
			buffer: HashMap::from([(
				Entity::from_raw(32),
				HashSet::from([ComponentString {
					comp: "A",
					dto: None,
					value: from_str(r#"{"value": 32}"#).unwrap(),
				}]),
			)]),
			writer: Mock_Writer::new_mock(|mock| {
				mock.expect_write().return_const(Ok(()));
			}),
			handlers: vec![],
		}));
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(SaveContext::write_file_system(context.clone()))?;

		assert_eq!(
			HashMap::default(),
			context.lock().expect("COULD NOT LOCK CONTEXT").buffer
		);
		Ok(())
	}
}

#[cfg(test)]
mod test_buffer {
	use super::*;
	use common::{simple_init, traits::mock::Mock};
	use mockall::{automock, predicate::eq};
	use serde_json::from_str;
	use std::path::PathBuf;

	#[automock]
	trait _Call {
		fn call(&self, buffer: &mut Buffer, entity: Entity) -> Result<(), Error>;
	}

	simple_init!(Mock_Call);

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn buffer() {
		fn get_buffer() -> Buffer {
			HashMap::from([(
				Entity::from_raw(42),
				HashSet::from([ComponentString {
					comp: "name",
					dto: None,
					value: from_str("[\"state\"]").unwrap(),
				}]),
			)])
		}

		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let entity = app.world().entity(entity);
		let mut context = SaveContext::from(FileWriter::to_destination(PathBuf::new()));
		context.buffer = get_buffer();
		context.handlers = vec![|b, e| {
			Mock_Call::new_mock(|mock| {
				mock.expect_call()
					.times(1)
					.with(eq(get_buffer()), eq(Entity::from_raw(0)))
					.returning(|_, _| Ok(()));
			})
			.call(b, e.id())
		}];

		let result = context.write_buffer(entity);

		assert!(result.is_ok());
	}

	#[test]
	fn buffer_error() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let entity = app.world().entity(entity);
		let mut context = SaveContext::from(FileWriter::to_destination(PathBuf::new()));
		context.handlers = vec![
			|b, e| {
				Mock_Call::new_mock(|mock| {
					mock.expect_call()
						.returning(|_, _| Err(serde::ser::Error::custom("NOPE, U LOSE")));
				})
				.call(b, e.id())
			},
			|b, e| {
				Mock_Call::new_mock(|mock| {
					mock.expect_call()
						.returning(|_, _| Err(serde::ser::Error::custom("NOPE, U LOSE AGAIN")));
				})
				.call(b, e.id())
			},
		];

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

#[cfg(test)]
mod test_handle {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;
	use serde::Serialize;
	use serde_json::{from_str, to_string};
	use std::any::type_name;

	struct _Writer;

	impl WriteFile for _Writer {
		type TError = ();

		fn write(&self, _: &str) -> Result<(), Self::TError> {
			panic!("SHOULD NOT BE CALLED");
		}
	}

	#[derive(Component, Serialize, Clone)]
	struct _A {
		value: i32,
	}

	#[derive(Serialize, Clone)]
	struct _ADto {
		value: String,
	}

	impl From<_A> for _ADto {
		fn from(_A { value }: _A) -> Self {
			Self {
				value: value.to_string(),
			}
		}
	}

	#[derive(Component, Serialize, Clone)]
	struct _B {
		v: i32,
	}

	#[derive(Component, Clone)]
	struct _Fail;

	impl Serialize for _Fail {
		fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer,
		{
			Err(serde::ser::Error::custom("Fool! I refuse serialization"))
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn serialize_component() {
		let mut app = setup();
		let mut buffer = HashMap::default();
		let entity = app.world_mut().spawn(_A { value: 42 }).id();
		let entity = app.world().entity(entity);

		_ = SaveContext::handle::<_A, _A>(&mut buffer, entity);

		assert_eq!(
			HashMap::from([(
				entity.id(),
				HashSet::from([ComponentString {
					comp: type_name::<_A>(),
					dto: None,
					value: from_str(&to_string(&_A { value: 42 }).unwrap()).unwrap()
				}])
			)]),
			buffer
		);
	}

	#[test]
	fn serialize_component_with_dto() {
		let mut app = setup();
		let mut buffer = HashMap::default();
		let entity = app.world_mut().spawn(_A { value: 42 }).id();
		let entity = app.world().entity(entity);

		_ = SaveContext::handle::<_A, _ADto>(&mut buffer, entity);

		assert_eq!(
			HashMap::from([(
				entity.id(),
				HashSet::from([ComponentString {
					comp: type_name::<_A>(),
					dto: Some(type_name::<_ADto>()),
					value: from_str(
						&to_string(&_ADto {
							value: "42".to_owned()
						})
						.unwrap()
					)
					.unwrap()
				}])
			)]),
			buffer
		);
	}

	#[test]
	fn serialize_multiple_components() {
		let mut app = setup();
		let mut buffer = HashMap::default();
		let entity = app.world_mut().spawn((_A { value: 42 }, _B { v: 11 })).id();
		let entity = app.world().entity(entity);

		_ = SaveContext::handle::<_A, _A>(&mut buffer, entity);
		_ = SaveContext::handle::<_B, _B>(&mut buffer, entity);

		assert_eq!(
			HashMap::from([(
				entity.id(),
				HashSet::from([
					ComponentString {
						comp: type_name::<_A>(),
						dto: None,
						value: from_str(&to_string(&_A { value: 42 }).unwrap()).unwrap()
					},
					ComponentString {
						comp: type_name::<_B>(),
						dto: None,
						value: from_str(&to_string(&_B { v: 11 }).unwrap()).unwrap()
					}
				])
			)]),
			buffer
		);
	}

	#[test]
	fn ok() {
		let mut app = setup();
		let mut buffer = HashMap::default();
		let entity = app.world_mut().spawn(_A { value: 42 }).id();
		let entity = app.world().entity(entity);

		let result = SaveContext::handle::<_A, _A>(&mut buffer, entity);

		assert!(result.is_ok());
	}

	#[test]
	fn error() {
		let mut app = setup();
		let mut buffer = HashMap::default();
		let entity = app.world_mut().spawn(_Fail).id();
		let entity = app.world().entity(entity);

		let result = SaveContext::handle::<_Fail, _Fail>(&mut buffer, entity);

		assert_eq!(
			Err("Fool! I refuse serialization".to_owned()),
			result.map_err(|e| e.to_string())
		);
	}
}
