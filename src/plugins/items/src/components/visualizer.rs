use crate::traits::{entity_names::ViewEntityNames, view::ItemView};
use bevy::prelude::*;
use common::traits::{
	accessors::get::GetRef,
	track::{IsTracking, Track, Untrack},
};
use std::{collections::HashMap, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Visualizer<TView, TKey> {
	pub(crate) entities: HashMap<Name, Entity>,
	phantom_data: PhantomData<(TView, TKey)>,
}

impl<TView, TKey> Visualizer<TView, TKey> {
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(entities: [(Name, Entity); N]) -> Self {
		Self {
			entities: HashMap::from(entities),
			..default()
		}
	}
}

impl<TView, TKey> Default for Visualizer<TView, TKey> {
	fn default() -> Self {
		Self {
			entities: HashMap::new(),
			phantom_data: PhantomData,
		}
	}
}

impl<TView, TKey> Track<Name> for Visualizer<TView, TKey>
where
	TView: ViewEntityNames<TKey>,
{
	fn track(&mut self, entity: Entity, name: &Name) {
		if !TView::view_entity_names().contains(&name.as_str()) {
			return;
		}

		self.entities.insert(name.clone(), entity);
	}
}

impl<TView, TKey> IsTracking<Name> for Visualizer<TView, TKey> {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.entities.values().any(|e| e == entity)
	}
}

impl<TView, TKey> Untrack<Name> for Visualizer<TView, TKey> {
	fn untrack(&mut self, entity: &Entity) {
		self.entities.retain(|_, e| e != entity);
	}
}

impl<TView, TKey> GetRef<TKey, Entity> for Visualizer<TView, TKey>
where
	TView: ItemView<TKey>,
{
	fn get(&self, key: &TKey) -> Option<&Entity> {
		self.entities.get(&TView::view_entity_name(key).into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug, PartialEq)]
	struct _View;

	#[derive(Debug, PartialEq)]
	struct _Key;

	impl ViewEntityNames<_Key> for _View {
		fn view_entity_names() -> Vec<&'static str> {
			vec!["A", "B", "C"]
		}
	}

	#[test]
	fn track_name_if_contained_in_sub_model_names() {
		let mut lookup = Visualizer::<_View, _Key>::default();

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
		let mut lookup = Visualizer::<_View, _Key>::default();

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
		let mut lookup = Visualizer::<_View, _Key>::default();

		lookup.track(Entity::from_raw(33), &Name::from("A"));

		assert!(lookup.is_tracking(&Entity::from_raw(33)));
	}

	#[test]
	fn is_tracking_false() {
		let mut lookup = Visualizer::<_View, _Key>::default();

		lookup.track(Entity::from_raw(34), &Name::from("A"));

		assert!(!lookup.is_tracking(&Entity::from_raw(33)));
	}

	#[test]
	fn untrack() {
		let mut lookup = Visualizer::<_View, _Key>::default();

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

	#[test]
	fn get_entity_by_key() {
		struct _View;

		impl ItemView<_Key> for _View {
			type TFilter = ();
			type TViewComponents = ();

			fn view_entity_name(_: &_Key) -> &'static str {
				"A"
			}
		}

		let lookup = Visualizer::<_View, _Key>::new([(Name::from("A"), Entity::from_raw(100))]);

		assert_eq!(Some(&Entity::from_raw(100)), lookup.get(&_Key));
	}

	#[test]
	fn get_entity_by_other_key() {
		struct _View;

		impl ItemView<_Key> for _View {
			type TFilter = ();
			type TViewComponents = ();

			fn view_entity_name(_: &_Key) -> &'static str {
				"B"
			}
		}

		let lookup = Visualizer::<_View, _Key>::new([(Name::from("B"), Entity::from_raw(100))]);

		assert_eq!(Some(&Entity::from_raw(100)), lookup.get(&_Key));
	}
}
