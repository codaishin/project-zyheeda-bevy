use crate::traits::accessors::get::{GetField, Getter};
use bevy::{
	ecs::query::{QueryData, QueryFilter, ROQueryItem},
	prelude::*,
};
use uuid::Uuid;

/// A lookup key for queries.
///
/// It serves as a consistent way to store entity references throughout multiple game sessions.
/// This is achieved via the following heuristics;
///
/// - An internal [`Entity`] field is always treated as [`None`] for serialization/deserialization.
/// - When using [`Query::get_via_id`]:
///   - uses internal [`Entity`] for performant lookup
///   - uses internal id, if internal [`Entity`] is [`None`]
/// - Updates the internal [`Entity`] field when:
///   - added to an entity (requires [`crate::CommonPlugin`]).
///   - [`Query::get_via_id`] found an item.
#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ObjectId {
	id: Uuid,
	entity: Option<Entity>,
}

impl ObjectId {
	pub(crate) fn update_entity(mut entities: Query<(Entity, &mut Self), Changed<Self>>) {
		for (entity, mut id) in &mut entities {
			id.entity = Some(entity);
		}
	}
}

impl Default for ObjectId {
	/// Created with a new random internal id
	fn default() -> Self {
		Self {
			id: Uuid::new_v4(),
			entity: None,
		}
	}
}

pub trait GetThroughObjectId<D>
where
	D: QueryData,
{
	/// Get query item through a [`ObjectId`].
	///
	/// Updates the given key with the target [`ObjectId`].
	///
	/// <div class="warning">
	///   This is only implemented for a Query, if its first item is
	///   an ObjectId
	/// </div>
	fn get_via_id(&self, object_id: &mut ObjectId) -> Option<ROQueryItem<D>>;
}

impl<D, F> GetThroughObjectId<D> for Query<'_, '_, D, F>
where
	D: QueryData,
	F: QueryFilter,
	for<'w> ROQueryItem<'w, D>: Getter<ObjectId>,
{
	fn get_via_id(&self, object_id: &mut ObjectId) -> Option<ROQueryItem<D>> {
		let item = match object_id.entity {
			Some(entity) => self.get(entity).ok()?,
			None => self
				.iter()
				.find(|item| ObjectId::get_field(item).id == object_id.id)?,
		};

		*object_id = ObjectId::get_field(&item);
		Some(item)
	}
}

macro_rules! impl_get_object_id_peel {
	($name:ident, $($other:ident,)*) => (impl_get_object_id! { $($other,)* })
}

macro_rules! impl_get_object_id {
	() => (
		impl Getter<ObjectId> for (&ObjectId,) {
			fn get(&self) -> ObjectId {
				let (id, ..) = self;
				**id
			}
		}
	);
	( $($name:ident,)+ ) => (
		impl< $($name),+> Getter<ObjectId> for ( &ObjectId, $(&$name,)+) {
			fn get(&self) -> ObjectId {
				let (id, ..) = self;
				**id
			}
		}
		impl_get_object_id_peel! { $($name,)+ }
	)
}

impl_get_object_id! { A, B, C, D, E, F, }

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::SingleThreadedApp;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use std::sync::{Arc, Mutex};

	#[test]
	fn default_id_is_not_nil() {
		let ObjectId { id, .. } = ObjectId::default();

		assert!(id != Uuid::nil());
	}

	#[test]
	fn entity_is_none() {
		let ObjectId { entity, .. } = ObjectId::default();

		assert!(entity.is_none());
	}

	#[test]
	fn uuids_are_different() {
		let a = ObjectId::default();
		let b = ObjectId::default();

		assert!(
			a != b,
			"Expected left and right to be different, but they were equal\
			 \n  left: {a:?}\
			 \n right: {b:?}"
		);
	}

	#[test]
	fn test_get_object_id_from_query_item() {
		let id = ObjectId {
			id: Uuid::new_v4(),
			entity: Some(Entity::from_raw(33)),
		};

		assert_eq!(id, (&id,).get());
		assert_eq!(id, (&id, &3).get());
		assert_eq!(id, (&id, &3, &4).get());
		assert_eq!(id, (&id, &3, &4, &5).get());
		assert_eq!(id, (&id, &3, &4, &5, &6).get());
		assert_eq!(id, (&id, &3, &4, &5, &6, &7).get());
		assert_eq!(id, (&id, &3, &4, &5, &6, &7, &8).get());
	}

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	struct _Value(u8);

	#[test]
	fn get_entity_through_key_entity() -> Result<(), RunSystemError> {
		let mut app = App::new();
		let entity = app
			.world_mut()
			.spawn((
				_Value(42),
				ObjectId {
					id: Uuid::new_v4(),
					entity: None, // setting this none, so we force implementation to use Query::get()
				},
			))
			.id();
		let mut key = ObjectId {
			id: Uuid::new_v4(),
			entity: Some(entity),
		};

		let value = app
			.world_mut()
			.run_system_once(move |e: Query<(&ObjectId, &_Value)>| {
				e.get_via_id(&mut key).map(|(_, v)| *v)
			})?;

		assert_eq!(Some(_Value(42)), value);
		Ok(())
	}

	#[test]
	fn get_entity_when_key_id_matches_and_entity_is_none() -> Result<(), RunSystemError> {
		let mut app = App::new();
		let id = Uuid::new_v4();
		app.world_mut()
			.spawn((_Value(42), ObjectId { id, entity: None }));
		let mut key = ObjectId { id, entity: None };

		let value = app
			.world_mut()
			.run_system_once(move |e: Query<(&ObjectId, &_Value)>| {
				e.get_via_id(&mut key).map(|(_, v)| *v)
			})?;

		assert_eq!(Some(_Value(42)), value);
		Ok(())
	}

	#[test]
	fn return_none_when_object_id_missing() -> Result<(), RunSystemError> {
		let mut app = App::new();
		app.world_mut().spawn(_Value(42));
		let mut key = ObjectId {
			id: Uuid::new_v4(),
			entity: None,
		};

		let value = app
			.world_mut()
			.run_system_once(move |e: Query<(&ObjectId, &_Value)>| {
				e.get_via_id(&mut key).map(|(_, v)| *v)
			})?;

		assert_eq!(None, value);
		Ok(())
	}

	#[test]
	fn return_none_on_entity_mismatch() -> Result<(), RunSystemError> {
		let mut app = App::new();
		let id = Uuid::new_v4();
		app.world_mut().spawn((
			_Value(42),
			ObjectId {
				id,
				entity: None, // setting this none, so we force implementation to use Query::get()
			},
		));
		let mut key = ObjectId {
			id,
			entity: Some(Entity::from_raw(69)),
		};

		let value = app
			.world_mut()
			.run_system_once(move |e: Query<(&ObjectId, &_Value)>| {
				e.get_via_id(&mut key).map(|(_, v)| *v)
			})?;

		assert_eq!(None, value);
		Ok(())
	}

	#[test]
	fn update_given_key_entity() -> Result<(), RunSystemError> {
		let mut app = App::new();
		let id = Uuid::new_v4();
		let entity = app.world_mut().spawn(_Value(42)).id();
		app.world_mut().entity_mut(entity).insert(ObjectId {
			id,
			entity: Some(entity), // using target ObjectId's entity as source of truth
		});
		let key = Arc::new(Mutex::new(ObjectId { id, entity: None }));
		let key_handle = key.clone();

		app.world_mut()
			.run_system_once(move |e: Query<(&ObjectId, &_Value)>| {
				let mut key = key_handle.lock().unwrap();
				e.get_via_id(&mut key);
			})?;

		assert_eq!(Some(entity), key.lock().unwrap().entity);
		Ok(())
	}

	#[test]
	fn update_given_key_id() -> Result<(), RunSystemError> {
		let mut app = App::new();
		let id = Uuid::new_v4();
		let entity = app
			.world_mut()
			.spawn((
				_Value(42),
				ObjectId {
					id,
					entity: None, // setting this none, so we force implementation to use Query::get()
				},
			))
			.id();
		let key = Arc::new(Mutex::new(ObjectId {
			id: Uuid::new_v4(),
			entity: Some(entity),
		}));
		let key_handle = key.clone();

		app.world_mut()
			.run_system_once(move |e: Query<(&ObjectId, &_Value)>| {
				let mut key = key_handle.lock().unwrap();
				e.get_via_id(&mut key);
			})?;

		assert_eq!(id, key.lock().unwrap().id);
		Ok(())
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, ObjectId::update_entity);

		app
	}

	#[test]
	fn update_entity() {
		let mut app = setup();
		let id = Uuid::new_v4();
		let entity = app.world_mut().spawn(ObjectId { id, entity: None }).id();

		app.update();

		assert_eq!(
			Some(&ObjectId {
				id,
				entity: Some(entity)
			}),
			app.world().entity(entity).get::<ObjectId>()
		);
	}

	#[test]
	fn update_entity_only_once() {
		#[derive(Resource, Debug, PartialEq, Default)]
		struct _Changed(bool);

		impl _Changed {
			fn system(mut commands: Commands, changed: Query<(), Changed<ObjectId>>) {
				commands.insert_resource(_Changed(changed.iter().count() > 0));
			}
		}

		let mut app = setup();
		let id = Uuid::new_v4();
		app.world_mut().spawn(ObjectId { id, entity: None });
		app.init_resource::<_Changed>();
		app.add_systems(PostUpdate, _Changed::system);

		app.update();
		app.update();

		assert_eq!(&_Changed(false), app.world().resource::<_Changed>());
	}

	#[test]
	fn update_again_entity_on_change() {
		let mut app = setup();
		let id = Uuid::new_v4();
		let entity = app.world_mut().spawn(ObjectId { id, entity: None }).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<ObjectId>()
			.unwrap()
			.entity = Some(Entity::from_raw(1111));
		app.update();

		assert_eq!(
			Some(&ObjectId {
				id,
				entity: Some(entity)
			}),
			app.world().entity(entity).get::<ObjectId>()
		);
	}
}
