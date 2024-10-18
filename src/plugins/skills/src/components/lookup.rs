use bevy::prelude::*;
use common::traits::track::{IsTracking, Track, Untrack};
use std::{collections::HashMap, marker::PhantomData};

use crate::traits::entity_names::EntityNames;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Lookup<T> {
	models: HashMap<Name, Entity>,
	phantom_data: PhantomData<T>,
}

impl<T> Default for Lookup<T> {
	fn default() -> Self {
		Self {
			models: HashMap::new(),
			phantom_data: PhantomData,
		}
	}
}

impl<T> Track<Name> for Lookup<T>
where
	T: EntityNames,
{
	fn track(&mut self, entity: Entity, name: &Name) {
		if !T::entity_names().contains(&name.as_str()) {
			return;
		}

		self.models.insert(name.clone(), entity);
	}
}

impl<T> IsTracking<Name> for Lookup<T> {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.models.values().any(|e| e == entity)
	}
}

impl<T> Untrack<Name> for Lookup<T> {
	fn untrack(&mut self, entity: &Entity) {
		self.models.retain(|_, e| e != entity);
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
		let mut sub_models = Lookup::<_Agent>::default();

		sub_models.track(Entity::from_raw(33), &Name::from("A"));

		assert_eq!(
			Lookup {
				models: HashMap::from([(Name::from("A"), Entity::from_raw(33))]),
				..default()
			},
			sub_models,
		);
	}

	#[test]
	fn do_not_track_name_if_not_contained_in_sub_model_names() {
		let mut sub_models = Lookup::<_Agent>::default();

		sub_models.track(Entity::from_raw(33), &Name::from("D"));

		assert_eq!(
			Lookup {
				models: HashMap::from([]),
				..default()
			},
			sub_models,
		);
	}

	#[test]
	fn is_tracking_true() {
		let mut sub_models = Lookup::<_Agent>::default();

		sub_models.track(Entity::from_raw(33), &Name::from("A"));

		assert!(sub_models.is_tracking(&Entity::from_raw(33)));
	}

	#[test]
	fn is_tracking_false() {
		let mut sub_models = Lookup::<_Agent>::default();

		sub_models.track(Entity::from_raw(34), &Name::from("A"));

		assert!(!sub_models.is_tracking(&Entity::from_raw(33)));
	}

	#[test]
	fn untrack() {
		let mut sub_models = Lookup::<_Agent>::default();

		sub_models.track(Entity::from_raw(34), &Name::from("A"));
		sub_models.track(Entity::from_raw(35), &Name::from("B"));
		sub_models.untrack(&Entity::from_raw(34));

		assert_eq!(
			Lookup {
				models: HashMap::from([(Name::from("B"), Entity::from_raw(35))]),
				..default()
			},
			sub_models,
		);
	}
}
