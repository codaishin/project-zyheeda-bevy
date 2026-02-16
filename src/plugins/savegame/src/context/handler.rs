use crate::{
	context::{ComponentString, SaveBuffer},
	errors::SerdeJsonError,
	traits::{
		buffer_entity_component::BufferEntityComponent,
		insert_entity_component::InsertEntityComponent,
	},
};
use bevy::prelude::*;
use common::traits::{handles_saving::SavableComponent, load_asset::LoadAsset};
use serde_json::{Error, Value};
use std::{
	any::type_name,
	collections::{HashSet, hash_map::Entry},
};

#[derive(Debug, Clone)]
pub(crate) struct ComponentHandler<TLoadAsset = AssetServer> {
	buffer_fn: fn(&mut SaveBuffer, EntityRef) -> Result<(), Error>,
	insert_fn: fn(&mut EntityCommands, Value, &mut TLoadAsset) -> Result<(), SerdeJsonError>,
	component_name_fn: fn() -> &'static str,
}

impl<TLoadAsset> ComponentHandler<TLoadAsset>
where
	TLoadAsset: LoadAsset,
{
	pub(crate) fn new<T>() -> Self
	where
		T: SavableComponent,
	{
		Self {
			buffer_fn: Self::buffer::<T>,
			insert_fn: Self::insert::<T>,
			component_name_fn: || type_name::<T>(),
		}
	}

	fn buffer<T>(buffer: &mut SaveBuffer, entity: EntityRef) -> Result<(), Error>
	where
		T: SavableComponent,
	{
		let Some(component) = entity.get::<T>() else {
			return Ok(());
		};
		let component_str = ComponentString {
			comp: type_name::<T>().to_owned(),
			value: serde_json::to_value(T::TDto::from(component.clone()))?,
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

	fn insert<T>(
		entity: &mut EntityCommands,
		component: Value,
		asset_server: &mut TLoadAsset,
	) -> Result<(), SerdeJsonError>
	where
		T: SavableComponent,
	{
		let dto = serde_json::from_value::<T::TDto>(component).map_err(SerdeJsonError)?;
		let Ok(component) = T::try_load_from(dto, asset_server);

		entity.try_insert(component);

		Ok(())
	}
}

impl<TLoadAsset> PartialEq for ComponentHandler<TLoadAsset> {
	fn eq(&self, other: &Self) -> bool {
		std::ptr::fn_addr_eq(self.buffer_fn, other.buffer_fn)
			&& std::ptr::fn_addr_eq(self.insert_fn, other.insert_fn)
	}
}

impl<TLoadAsset> BufferEntityComponent for ComponentHandler<TLoadAsset>
where
	TLoadAsset: LoadAsset,
{
	fn buffer_component(&self, buffer: &mut SaveBuffer, entity: EntityRef) -> Result<(), Error> {
		(self.buffer_fn)(buffer, entity)
	}
}

impl<TLoadAsset> InsertEntityComponent<TLoadAsset> for ComponentHandler<TLoadAsset>
where
	TLoadAsset: LoadAsset,
{
	type TComponent = Value;
	type TError = SerdeJsonError;

	fn insert_component(
		&self,
		entity: &mut EntityCommands,
		components: Value,
		asset_server: &mut TLoadAsset,
	) -> Result<(), SerdeJsonError> {
		(self.insert_fn)(entity, components, asset_server)
	}

	fn component_name(&self) -> &'static str {
		(self.component_name_fn)()
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::traits::write_file::WriteFile;
	use bevy::asset::AssetPath;
	use common::{
		errors::Unreachable,
		traits::{handles_custom_assets::TryLoadFrom, load_asset::LoadAsset},
	};
	use macros::SavableComponent;
	use serde::{Deserialize, Serialize};
	use serde_json::{from_str, to_string};
	use std::{any::type_name, collections::HashMap};
	use testing::SingleThreadedApp;

	#[derive(Resource)]
	struct _LoadAsset;

	impl LoadAsset for _LoadAsset {
		fn load_asset<'a, TAsset, TPath>(&mut self, _: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'a>>,
		{
			panic!("NOT USED");
		}
	}

	struct _Writer;

	impl WriteFile for _Writer {
		type TWriteError = ();

		fn write(&self, _: &str) -> Result<(), Self::TWriteError> {
			panic!("NOT USED");
		}
	}

	#[derive(Component, SavableComponent, Clone, PartialEq, Debug)]
	#[savable_component(id = "a", dto = _ADto)]
	struct _A {
		value: i32,
	}

	#[derive(Serialize, Deserialize, Clone)]
	struct _ADto {
		value: u32,
	}

	impl From<_A> for _ADto {
		fn from(_A { value }: _A) -> Self {
			Self {
				value: value as u32,
			}
		}
	}

	impl TryLoadFrom<_ADto> for _A {
		type TInstantiationError = Unreachable;

		fn try_load_from<TLoadAsset>(
			_ADto { value }: _ADto,
			_: &mut TLoadAsset,
		) -> Result<Self, Unreachable> {
			Ok(Self {
				value: value as i32,
			})
		}
	}

	#[derive(Component, SavableComponent, Serialize, Deserialize, Clone, PartialEq, Debug)]
	#[savable_component(id = "b")]
	struct _B {
		v: i32,
	}

	#[derive(Component, SavableComponent, Clone)]
	#[savable_component(id = "fail")]
	struct _Fail;

	impl Serialize for _Fail {
		fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer,
		{
			Err(serde::ser::Error::custom("Fool! I refuse serialization"))
		}
	}

	impl<'de> Deserialize<'de> for _Fail {
		fn deserialize<D>(_: D) -> Result<Self, D::Error>
		where
			D: serde::Deserializer<'de>,
		{
			Err(serde::de::Error::custom("Fool! I refuse deserialization"))
		}
	}

	mod serialize {
		use super::*;

		fn setup() -> App {
			App::new().single_threaded(Update)
		}

		#[test]
		fn component_with_dto() {
			let mut app = setup();
			let mut buffer = HashMap::default();
			let entity = app.world_mut().spawn(_A { value: 42 }).id();
			let entity = app.world().entity(entity);

			_ = ComponentHandler::<_LoadAsset>::new::<_A>().buffer_component(&mut buffer, entity);

			assert_eq!(
				HashMap::from([(
					entity.id(),
					HashSet::from([ComponentString {
						comp: type_name::<_A>().to_owned(),
						value: from_str(&to_string(&_ADto { value: 42 }).unwrap()).unwrap()
					}])
				)]),
				buffer
			);
		}

		#[test]
		fn component_without_dto() {
			let mut app = setup();
			let mut buffer = HashMap::default();
			let entity = app.world_mut().spawn(_B { v: 42 }).id();
			let entity = app.world().entity(entity);

			_ = ComponentHandler::<_LoadAsset>::new::<_B>().buffer_component(&mut buffer, entity);

			assert_eq!(
				HashMap::from([(
					entity.id(),
					HashSet::from([ComponentString {
						comp: type_name::<_B>().to_owned(),
						value: from_str(&to_string(&_B { v: 42 }).unwrap()).unwrap()
					}])
				)]),
				buffer
			);
		}

		#[test]
		fn multiple_components() {
			let mut app = setup();
			let mut buffer = HashMap::default();
			let entity = app.world_mut().spawn((_A { value: 42 }, _B { v: 11 })).id();
			let entity = app.world().entity(entity);

			_ = ComponentHandler::<_LoadAsset>::new::<_A>().buffer_component(&mut buffer, entity);
			_ = ComponentHandler::<_LoadAsset>::new::<_B>().buffer_component(&mut buffer, entity);

			assert_eq!(
				HashMap::from([(
					entity.id(),
					HashSet::from([
						ComponentString {
							comp: type_name::<_A>().to_owned(),
							value: from_str(&to_string(&_ADto { value: 42 }).unwrap()).unwrap()
						},
						ComponentString {
							comp: type_name::<_B>().to_owned(),
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

			let result =
				ComponentHandler::<_LoadAsset>::new::<_A>().buffer_component(&mut buffer, entity);

			assert!(result.is_ok());
		}

		#[test]
		fn error() {
			let mut app = setup();
			let mut buffer = HashMap::default();
			let entity = app.world_mut().spawn(_Fail).id();
			let entity = app.world().entity(entity);

			let result = ComponentHandler::<_LoadAsset>::new::<_Fail>()
				.buffer_component(&mut buffer, entity);

			assert_eq!(
				Err("Fool! I refuse serialization".to_owned()),
				result.map_err(|e| e.to_string())
			);
		}
	}

	mod deserialize {
		use super::*;
		use bevy::ecs::system::{RunSystemError, RunSystemOnce};
		use serde_json::json;

		#[derive(Component, SavableComponent, Serialize, Deserialize, Clone, PartialEq, Debug)]
		#[savable_component(id = "c")]
		struct _C {
			v: i32,
		}

		fn setup() -> App {
			let mut app = App::new().single_threaded(Update);

			app.insert_resource(_LoadAsset);

			app
		}

		#[test]
		fn component_with_dto() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();
			let handler = ComponentHandler::<_LoadAsset>::new::<_A>();

			_ = app.world_mut().run_system_once(
				move |mut commands: Commands, mut asset_server: ResMut<_LoadAsset>| {
					let entity = &mut commands.entity(entity);
					let asset_server = asset_server.as_mut();

					handler.insert_component(entity, json!({"value": 42}), asset_server)
				},
			)?;

			assert_eq!(
				Some(&_A { value: 42 }),
				app.world().entity(entity).get::<_A>(),
			);
			Ok(())
		}

		#[test]
		fn component_without_dto() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();
			let handler = ComponentHandler::<_LoadAsset>::new::<_B>();

			_ = app.world_mut().run_system_once(
				move |mut commands: Commands, mut asset_server: ResMut<_LoadAsset>| {
					let entity = &mut commands.entity(entity);
					let asset_server = asset_server.as_mut();

					handler.insert_component(entity, json!({"v": 42}), asset_server)
				},
			)?;

			assert_eq!(Some(&_B { v: 42 }), app.world().entity(entity).get::<_B>());
			Ok(())
		}

		#[test]
		fn return_errors() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();
			let handler = ComponentHandler::<_LoadAsset>::new::<_Fail>();

			let result = app.world_mut().run_system_once(
				move |mut commands: Commands, mut asset_server: ResMut<_LoadAsset>| {
					let entity = &mut commands.entity(entity);
					let asset_server = asset_server.as_mut();

					handler.insert_component(entity, json!("{v: 42}"), asset_server)
				},
			)?;

			assert_eq!(
				Err(SerdeJsonError(serde::de::Error::custom(
					"Fool! I refuse deserialization"
				))),
				result
			);
			Ok(())
		}

		#[test]
		fn get_component_name() {
			let handler = ComponentHandler::<_LoadAsset>::new::<_A>();

			assert_eq!(type_name::<_A>(), handler.component_name());
		}
	}
}
