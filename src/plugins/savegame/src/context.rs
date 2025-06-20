use crate::{
	errors::EntitySerializationErrors,
	file_io::FileIO,
	traits::write_buffer::WriteBuffer,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Error, Value, to_value};
use std::{
	any::type_name,
	collections::{HashMap, HashSet, hash_map::Entry},
};

pub(crate) type SaveBuffer = HashMap<Entity, HashSet<ComponentString>>;
pub(crate) type LoadBuffer = Vec<HashSet<ComponentString>>;
pub(crate) type Handlers = Vec<fn(&mut SaveBuffer, EntityRef) -> Result<(), Error>>;

#[derive(Debug, PartialEq, Default)]
pub struct SaveContext<TFileIO = FileIO> {
	pub(crate) handlers: Handlers,
	pub(crate) save_buffer: SaveBuffer,
	pub(crate) load_buffer: LoadBuffer,
	pub(crate) io: TFileIO,
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
			comp: comp.to_owned(),
			dto: match type_name::<TDto>() {
				dto if dto == comp => None,
				dto => Some(dto.to_owned()),
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

#[cfg(test)]
impl<TFileIO> SaveContext<TFileIO> {
	pub(crate) fn with_save_buffer<const N: usize>(
		mut self,
		buffer: [(Entity, HashSet<ComponentString>); N],
	) -> Self {
		self.save_buffer = HashMap::from(buffer);
		self
	}
}

impl<TFileWriter> From<TFileWriter> for SaveContext<TFileWriter> {
	fn from(writer: TFileWriter) -> Self {
		Self {
			io: writer,
			handlers: vec![],
			load_buffer: vec![],
			save_buffer: HashMap::default(),
		}
	}
}

impl WriteBuffer for SaveContext {
	fn write_buffer(&mut self, entity: EntityRef) -> Result<(), EntitySerializationErrors> {
		let errors = self
			.handlers
			.iter()
			.filter_map(|handler| handler(&mut self.save_buffer, entity).err())
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

	/// Component dto type name, abbreviated to reduce memory usage
	#[serde(skip_serializing_if = "Option::is_none")]
	pub(crate) dto: Option<String>,

	/// Component serialized value
	pub(crate) value: Value,
}

#[cfg(test)]
mod test_write_buffer {
	use super::*;
	use common::{simple_init, traits::mock::Mock};
	use mockall::{automock, predicate::eq};
	use serde_json::from_str;
	use std::path::PathBuf;

	#[automock]
	trait _Call {
		fn call(&self, buffer: &mut SaveBuffer, entity: Entity) -> Result<(), Error>;
	}

	simple_init!(Mock_Call);

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn buffer() {
		fn get_buffer() -> SaveBuffer {
			HashMap::from([(
				Entity::from_raw(42),
				HashSet::from([ComponentString {
					comp: "name".to_owned(),
					dto: None,
					value: from_str("[\"state\"]").unwrap(),
				}]),
			)])
		}

		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let entity = app.world().entity(entity);
		let mut context = SaveContext::from(FileIO::with_file(PathBuf::new()));
		context.save_buffer = get_buffer();
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
		let mut context = SaveContext::from(FileIO::with_file(PathBuf::new()));
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
	use crate::traits::write_file::WriteFile;
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
					comp: type_name::<_A>().to_owned(),
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
					comp: type_name::<_A>().to_owned(),
					dto: Some(type_name::<_ADto>().to_owned()),
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
						comp: type_name::<_A>().to_owned(),
						dto: None,
						value: from_str(&to_string(&_A { value: 42 }).unwrap()).unwrap()
					},
					ComponentString {
						comp: type_name::<_B>().to_owned(),
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
