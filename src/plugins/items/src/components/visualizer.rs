use crate::traits::{entity_names::EntityNames, key_string::KeyString};
use bevy::prelude::*;
use common::traits::{
	accessors::get::GetRef,
	track::{IsTracking, Track, Untrack},
};
use std::{collections::HashMap, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub struct Visualizer<T> {
	pub(crate) entities: HashMap<Name, Entity>,
	phantom_data: PhantomData<T>,
}

impl<T> Visualizer<T> {
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(entities: [(Name, Entity); N]) -> Self {
		Self {
			entities: HashMap::from(entities),
			..default()
		}
	}
}

impl<T> Default for Visualizer<T> {
	fn default() -> Self {
		Self {
			entities: HashMap::new(),
			phantom_data: PhantomData,
		}
	}
}

impl<T> Track<Name> for Visualizer<T>
where
	T: EntityNames,
{
	fn track(&mut self, entity: Entity, name: &Name) {
		if !T::entity_names().contains(&name.as_str()) {
			return;
		}

		self.entities.insert(name.clone(), entity);
	}
}

impl<T> IsTracking<Name> for Visualizer<T> {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.entities.values().any(|e| e == entity)
	}
}

impl<T> Untrack<Name> for Visualizer<T> {
	fn untrack(&mut self, entity: &Entity) {
		self.entities.retain(|_, e| e != entity);
	}
}

impl<T, TKey> GetRef<TKey, Entity> for Visualizer<T>
where
	T: KeyString<TKey>,
{
	fn get(&self, key: &TKey) -> Option<&Entity> {
		self.entities.get(&Name::from(T::key_string(key)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug, PartialEq)]
	struct _Agent;

	impl EntityNames for _Agent {
		fn entity_names() -> Vec<&'static str> {
			vec!["A", "B", "C"]
		}
	}

	#[test]
	fn track_name_if_contained_in_sub_model_names() {
		let mut lookup = Visualizer::<_Agent>::default();

		lookup.track(Entity::from_raw(33), &Name::from("A"));

		assert_eq!(
			Visualizer {
				entities: HashMap::from([(Name::from("A"), Entity::from_raw(33))]),
				..default()
			},
			lookup,
		);
	}

	#[test]
	fn do_not_track_name_if_not_contained_in_sub_model_names() {
		let mut lookup = Visualizer::<_Agent>::default();

		lookup.track(Entity::from_raw(33), &Name::from("D"));

		assert_eq!(
			Visualizer {
				entities: HashMap::from([]),
				..default()
			},
			lookup,
		);
	}

	#[test]
	fn is_tracking_true() {
		let mut lookup = Visualizer::<_Agent>::default();

		lookup.track(Entity::from_raw(33), &Name::from("A"));

		assert!(lookup.is_tracking(&Entity::from_raw(33)));
	}

	#[test]
	fn is_tracking_false() {
		let mut lookup = Visualizer::<_Agent>::default();

		lookup.track(Entity::from_raw(34), &Name::from("A"));

		assert!(!lookup.is_tracking(&Entity::from_raw(33)));
	}

	#[test]
	fn untrack() {
		let mut lookup = Visualizer::<_Agent>::default();

		lookup.track(Entity::from_raw(34), &Name::from("A"));
		lookup.track(Entity::from_raw(35), &Name::from("B"));
		lookup.untrack(&Entity::from_raw(34));

		assert_eq!(
			Visualizer {
				entities: HashMap::from([(Name::from("B"), Entity::from_raw(35))]),
				..default()
			},
			lookup,
		);
	}

	struct _Key;

	#[test]
	fn get_entity_by_key() {
		struct _T;

		impl KeyString<_Key> for _T {
			fn key_string(_: &_Key) -> &'static str {
				"A"
			}
		}

		let lookup = Visualizer::<_T>::new([(Name::from("A"), Entity::from_raw(100))]);

		assert_eq!(Some(&Entity::from_raw(100)), lookup.get(&_Key));
	}

	#[test]
	fn get_entity_by_other_key() {
		struct _T;

		impl KeyString<_Key> for _T {
			fn key_string(_: &_Key) -> &'static str {
				"B"
			}
		}

		let lookup = Visualizer::<_T>::new([(Name::from("B"), Entity::from_raw(100))]);

		assert_eq!(Some(&Entity::from_raw(100)), lookup.get(&_Key));
	}
}
