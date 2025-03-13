use crate::{
	errors::{IoOrLockError, LockPoisonedError},
	traits::{execute_save::BufferComponents, write_to_file::WriteToFile},
	writer::FileWriter,
};
use bevy::prelude::*;
use serde::Serialize;
use serde_json::to_string;
use std::{
	any::type_name,
	collections::{HashMap, HashSet, hash_map::Entry},
	sync::{Arc, Mutex},
};

pub(crate) type Buffer = HashMap<Entity, HashSet<ComponentString>>;
pub(crate) type Handlers = Vec<fn(&mut Buffer, EntityRef)>;

#[derive(Debug, PartialEq, Default)]
pub struct SaveContext<TFileWriter = FileWriter> {
	pub(crate) handlers: Handlers,
	writer: TFileWriter,
	buffer: Buffer,
}

impl SaveContext {
	pub(crate) fn handle<T>(
		buffer: &mut HashMap<Entity, HashSet<ComponentString>>,
		entity: EntityRef,
	) where
		T: Component + Serialize,
	{
		let Some(component) = entity.get::<T>() else {
			return;
		};
		let component_str = ComponentString {
			component_name: type_name::<T>(),
			component_state: to_string(component).unwrap(),
		};

		match buffer.entry(entity.id()) {
			Entry::Occupied(mut occupied_entry) => {
				occupied_entry.get_mut().insert(component_str);
			}
			Entry::Vacant(vacant_entry) => {
				vacant_entry.insert(HashSet::from([component_str]));
			}
		}
	}
}

impl<TFileWriter> SaveContext<TFileWriter> {
	pub(crate) fn new(writer: TFileWriter) -> Self {
		Self {
			writer,
			handlers: vec![],
			buffer: HashMap::default(),
		}
	}

	pub(crate) fn flush_system(
		context: Arc<Mutex<Self>>,
	) -> impl Fn() -> Result<(), IoOrLockError<TFileWriter::TError>>
	where
		TFileWriter: WriteToFile,
	{
		move || {
			let mut context = match context.lock() {
				Err(_) => return Err(IoOrLockError::LockPoisoned(LockPoisonedError)),
				Ok(context) => context,
			};

			match context.flush() {
				Err(io_error) => Err(IoOrLockError::IoError(io_error)),
				Ok(()) => Ok(()),
			}
		}
	}

	fn flush(&mut self) -> Result<(), TFileWriter::TError>
	where
		TFileWriter: WriteToFile,
	{
		let entities = self
			.buffer
			.drain()
			.map(join_entity_components)
			.collect::<Vec<_>>()
			.join(",");

		self.writer.write(format!("[{entities}]"))
	}
}

fn join_entity_components((_, component_strings): (Entity, HashSet<ComponentString>)) -> String {
	let components = component_strings
		.iter()
		.filter_map(|v| to_string(&v).ok())
		.collect::<Vec<_>>()
		.join(",");

	format!("[{components}]")
}

impl<TFileWriter> BufferComponents for SaveContext<TFileWriter>
where
	TFileWriter: WriteToFile,
{
	fn buffer_components(&mut self, entity: EntityRef) {
		for handler in &self.handlers {
			handler(&mut self.buffer, entity);
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Clone)]
pub(crate) struct ComponentString {
	component_name: &'static str,
	component_state: String,
}

#[cfg(test)]
mod test_flush {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{simple_init, test_tools::utils::SingleThreadedApp, traits::mock::Mock};
	use mockall::{mock, predicate::eq};

	#[derive(Debug, PartialEq, Clone)]
	struct _Error;

	mock! {
	  _Writer {}
		impl WriteToFile for _Writer {
			type TError = _Error;
			fn write(&self, string: String) -> Result<(), _Error>;
		}
	}

	simple_init!(Mock_Writer);

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn write_on_flush() -> Result<(), RunSystemError> {
		let string_a = ComponentString {
			component_name: "A",
			component_state: r#"{"value": 32}"#.to_owned(),
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
			.run_system_once(SaveContext::flush_system(context))?;
		Ok(())
	}

	#[test]
	fn write_multiple_components_per_entity_on_flush() -> Result<(), RunSystemError> {
		let string_a = ComponentString {
			component_name: "A",
			component_state: r#"{"value": 32}"#.to_owned(),
		};
		let string_b = ComponentString {
			component_name: "B",
			component_state: r#"{"v": 42}"#.to_owned(),
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
								component_name: "A",
								component_state: r#"{"value": 32}"#.to_owned(),
							})
							.unwrap(),
							to_string(&ComponentString {
								component_name: "B",
								component_state: r#"{"v": 42}"#.to_owned(),
							})
							.unwrap(),
						);
						let b_a = format!(
							"[[{},{}]]",
							to_string(&ComponentString {
								component_name: "B",
								component_state: r#"{"v": 42}"#.to_owned(),
							})
							.unwrap(),
							to_string(&ComponentString {
								component_name: "A",
								component_state: r#"{"value": 32}"#.to_owned(),
							})
							.unwrap(),
						);
						v == &a_b || v == &b_a
					})
					.return_const(Ok(()));
			}),
			handlers: vec![],
		}));
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(SaveContext::flush_system(context))?;
		Ok(())
	}

	#[test]
	fn write_multiple_entities_on_flush() -> Result<(), RunSystemError> {
		let string_a = ComponentString {
			component_name: "A",
			component_state: r#"{"value": 32}"#.to_owned(),
		};
		let string_b = ComponentString {
			component_name: "B",
			component_state: r#"{"v": 42}"#.to_owned(),
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
								component_name: "A",
								component_state: r#"{"value": 32}"#.to_owned(),
							})
							.unwrap(),
							to_string(&ComponentString {
								component_name: "B",
								component_state: r#"{"v": 42}"#.to_owned(),
							})
							.unwrap(),
						);
						let b_a = format!(
							"[[{}],[{}]]",
							to_string(&ComponentString {
								component_name: "B",
								component_state: r#"{"v": 42}"#.to_owned(),
							})
							.unwrap(),
							to_string(&ComponentString {
								component_name: "A",
								component_state: r#"{"value": 32}"#.to_owned(),
							})
							.unwrap(),
						);
						v == &a_b || v == &b_a
					})
					.return_const(Ok(()));
			}),
			handlers: vec![],
		}));
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(SaveContext::flush_system(context))?;
		Ok(())
	}

	#[test]
	fn clear_buffer_on_flush() -> Result<(), RunSystemError> {
		let context = Arc::new(Mutex::new(SaveContext {
			buffer: HashMap::from([(
				Entity::from_raw(32),
				HashSet::from([ComponentString {
					component_name: "A",
					component_state: r#"{"value": 32}"#.to_owned(),
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
			.run_system_once(SaveContext::flush_system(context.clone()))?;

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
	use common::test_tools::utils::SingleThreadedApp;
	use serde::Serialize;
	use std::any::type_name;

	struct _Writer;

	impl WriteToFile for _Writer {
		type TError = ();

		fn write(&self, _: String) -> Result<(), Self::TError> {
			panic!("SHOULD NOT BE CALLED");
		}
	}

	#[derive(Component, Serialize)]
	struct _A {
		value: i32,
	}

	#[derive(Component, Serialize)]
	struct _B {
		v: i32,
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

		SaveContext::handle::<_A>(&mut buffer, entity);

		assert_eq!(
			HashMap::from([(
				entity.id(),
				HashSet::from([ComponentString {
					component_name: type_name::<_A>(),
					component_state: to_string(&_A { value: 42 }).unwrap()
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

		SaveContext::handle::<_A>(&mut buffer, entity);
		SaveContext::handle::<_B>(&mut buffer, entity);

		assert_eq!(
			HashMap::from([(
				entity.id(),
				HashSet::from([
					ComponentString {
						component_name: type_name::<_A>(),
						component_state: to_string(&_A { value: 42 }).unwrap()
					},
					ComponentString {
						component_name: type_name::<_B>(),
						component_state: to_string(&_B { v: 11 }).unwrap()
					}
				])
			)]),
			buffer
		);
	}
}
