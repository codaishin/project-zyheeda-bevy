use crate::traits::accessors::get::{Getter, GetterRef};
use bevy::prelude::*;
use uuid::Uuid;

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ObjectId(Uuid);

impl Default for ObjectId {
	fn default() -> Self {
		Self(Uuid::new_v4())
	}
}

macro_rules! impl_get_object_id_peel {
	($name:ident, $($other:ident,)*) => (impl_get_object_id! { $($other,)* })
}

macro_rules! impl_get_object_id {
	() => (
		impl<TFst> GetterRef<ObjectId> for (TFst, &ObjectId) {
			fn get(&self) -> &ObjectId {
				let (_, id, ..) = self;
				id
			}
		}
	);
	( $($name:ident,)+ ) => (
		impl<TFst, $($name),+> GetterRef<ObjectId> for (TFst, &ObjectId, $(&$name,)+) {
			fn get(&self) -> &ObjectId {
				let (_, id, ..) = self;
				id
			}
		}
		impl_get_object_id_peel! { $($name,)+ }
	)
}

impl_get_object_id! { A, B, C, D, E, F, }

macro_rules! impl_get_entity_peel {
	($name:ident, $($other:ident,)*) => (impl_get_entity! { $($other,)* })
}

macro_rules! impl_get_entity {
	() => (
		impl<TSnd> Getter<Entity> for (Entity, &TSnd) {
			fn get(&self) -> Entity {
				let (entity, ..) = self;
				*entity
			}
		}
	);
	( $($name:ident,)+ ) => (
		impl<TSnd, $($name),+> Getter<Entity> for (Entity, &TSnd, $(&$name,)+) {
			fn get(&self) -> Entity {
				let (entity, ..) = self;
				*entity
			}
		}
		impl_get_entity_peel! { $($name,)+ }
	)
}

impl_get_entity! { A, B, C, D, E, F, }

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn default_is_not_nil() {
		let ObjectId(id) = ObjectId::default();

		assert!(id != Uuid::nil());
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
		let id = ObjectId(Uuid::new_v4());

		assert_eq!(&id, (&1, &id).get());
		assert_eq!(&id, (&1, &id, &3).get());
		assert_eq!(&id, (&1, &id, &3, &4).get());
		assert_eq!(&id, (&1, &id, &3, &4, &5).get());
		assert_eq!(&id, (&1, &id, &3, &4, &5, &6).get());
		assert_eq!(&id, (&1, &id, &3, &4, &5, &6, &7).get());
		assert_eq!(&id, (&1, &id, &3, &4, &5, &6, &7, &8).get());
	}

	#[test]
	fn test_get_entity_from_query_item() {
		let entity = Entity::from_raw(42);

		assert_eq!(entity, (entity, &2).get());
		assert_eq!(entity, (entity, &2, &3).get());
		assert_eq!(entity, (entity, &2, &3, &4).get());
		assert_eq!(entity, (entity, &2, &3, &4, &5).get());
		assert_eq!(entity, (entity, &2, &3, &4, &5, &6).get());
		assert_eq!(entity, (entity, &2, &3, &4, &5, &6, &7).get());
		assert_eq!(entity, (entity, &2, &3, &4, &5, &6, &7, &8).get());
	}
}
