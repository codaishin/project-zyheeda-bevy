use bevy::prelude::*;
use common::traits::track::{IsTracking, Track, Untrack};
use std::{collections::HashMap, marker::PhantomData};

use crate::traits::sub_model_names::SubModelNames;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct SubModels<TAgent> {
	models: HashMap<Name, Entity>,
	phantom_data: PhantomData<TAgent>,
}

impl<TAgent> Default for SubModels<TAgent> {
	fn default() -> Self {
		Self {
			models: HashMap::new(),
			phantom_data: PhantomData,
		}
	}
}

impl<TAgent> Track<Name> for SubModels<TAgent>
where
	TAgent: SubModelNames,
{
	fn track(&mut self, entity: Entity, name: &Name) {
		if !TAgent::sub_model_names().contains(&name.as_str()) {
			return;
		}

		self.models.insert(name.clone(), entity);
	}
}

impl<TAgent> IsTracking<Name> for SubModels<TAgent> {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.models.values().any(|e| e == entity)
	}
}

impl<TAgent> Untrack<Name> for SubModels<TAgent> {
	fn untrack(&mut self, entity: &Entity) {
		self.models.retain(|_, e| e != entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug, PartialEq)]
	struct _Agent;

	impl SubModelNames for _Agent {
		fn sub_model_names() -> Vec<&'static str> {
			vec!["A", "B", "C"]
		}
	}

	#[test]
	fn track_name_if_contained_in_sub_model_names() {
		let mut sub_models = SubModels::<_Agent>::default();

		sub_models.track(Entity::from_raw(33), &Name::from("A"));

		assert_eq!(
			SubModels {
				models: HashMap::from([(Name::from("A"), Entity::from_raw(33))]),
				..default()
			},
			sub_models,
		);
	}

	#[test]
	fn do_not_track_name_if_not_contained_in_sub_model_names() {
		let mut sub_models = SubModels::<_Agent>::default();

		sub_models.track(Entity::from_raw(33), &Name::from("D"));

		assert_eq!(
			SubModels {
				models: HashMap::from([]),
				..default()
			},
			sub_models,
		);
	}

	#[test]
	fn is_tracking_true() {
		let mut sub_models = SubModels::<_Agent>::default();

		sub_models.track(Entity::from_raw(33), &Name::from("A"));

		assert!(sub_models.is_tracking(&Entity::from_raw(33)));
	}

	#[test]
	fn is_tracking_false() {
		let mut sub_models = SubModels::<_Agent>::default();

		sub_models.track(Entity::from_raw(34), &Name::from("A"));

		assert!(!sub_models.is_tracking(&Entity::from_raw(33)));
	}

	#[test]
	fn untrack() {
		let mut sub_models = SubModels::<_Agent>::default();

		sub_models.track(Entity::from_raw(34), &Name::from("A"));
		sub_models.track(Entity::from_raw(35), &Name::from("B"));
		sub_models.untrack(&Entity::from_raw(34));

		assert_eq!(
			SubModels {
				models: HashMap::from([(Name::from("B"), Entity::from_raw(35))]),
				..default()
			},
			sub_models,
		);
	}
}
