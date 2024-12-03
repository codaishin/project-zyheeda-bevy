use bevy::prelude::{Component, Entity, Name, Query};
use common::traits::{
	accessors::get::GetRef,
	iteration::IterFinite,
	register_visualization::ContainsVisibleItemAssets,
	track::{IsTracking, Track, Untrack},
};
use loading::resources::track::Loaded;
use std::{collections::HashMap, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Visualize<TItemContainer, TMarker> {
	pub(crate) entities: HashMap<Name, Entity>,
	phantom_data: PhantomData<(TItemContainer, TMarker)>,
}

impl<TItemContainer, TMarker> Visualize<TItemContainer, TMarker>
where
	TItemContainer: ContainsVisibleItemAssets<TMarker> + Sync + Send + 'static,
	TItemContainer::TKey: IterFinite,
	TMarker: Sync + Send + 'static,
{
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(entities: [(Name, Entity); N]) -> Self {
		use bevy::utils::default;

		Self {
			entities: HashMap::from(entities),
			..default()
		}
	}

	pub(crate) fn entities_loaded(visualizers: Query<&Self>) -> Loaded {
		let key_count = TItemContainer::TKey::iterator().count();
		Loaded(
			visualizers
				.iter()
				.all(|visualizer| visualizer.entities.len() == key_count),
		)
	}
}

impl<TItemContainer, TMarker> Default for Visualize<TItemContainer, TMarker> {
	fn default() -> Self {
		Self {
			entities: HashMap::new(),
			phantom_data: PhantomData,
		}
	}
}

impl<TItemContainer, TMarker> Track<Name> for Visualize<TItemContainer, TMarker>
where
	TItemContainer: ContainsVisibleItemAssets<TMarker>,
	TItemContainer::TKey: IterFinite,
{
	fn track(&mut self, entity: Entity, name: &Name) {
		let entity_keys = TItemContainer::TKey::iterator();
		let entity_not_valid = !entity_keys
			.map(|key| TItemContainer::visualization_entity_name(&key))
			.any(|entity_name| entity_name == name.as_str());

		if entity_not_valid {
			return;
		}

		self.entities.insert(name.clone(), entity);
	}
}

impl<TContainer, TMarker> IsTracking<Name> for Visualize<TContainer, TMarker> {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.entities.values().any(|e| e == entity)
	}
}

impl<TContainer, TMarker> Untrack<Name> for Visualize<TContainer, TMarker> {
	fn untrack(&mut self, entity: &Entity) {
		self.entities.retain(|_, e| e != entity);
	}
}

impl<TContainer, TMarker> GetRef<TContainer::TKey, Entity> for Visualize<TContainer, TMarker>
where
	TContainer: ContainsVisibleItemAssets<TMarker>,
{
	fn get(&self, key: &TContainer::TKey) -> Option<&Entity> {
		self.entities
			.get(&TContainer::visualization_entity_name(key).into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{ecs::system::RunSystemOnce, prelude::*};
	use common::traits::{get_asset::GetAsset, iteration::Iter};

	#[derive(Component, Debug, PartialEq)]
	struct _ItemContainer;

	#[derive(Debug, PartialEq)]
	struct _View;

	#[derive(Asset, TypePath)]
	struct _Asset;

	#[derive(Debug, PartialEq, Clone, Copy)]
	enum _Key {
		A,
		B,
		C,
	}

	impl IterFinite for _Key {
		fn iterator() -> Iter<Self> {
			Iter(Some(_Key::A))
		}

		fn next(Iter(current): &Iter<Self>) -> Option<Self> {
			match current.as_ref()? {
				_Key::A => Some(_Key::B),
				_Key::B => Some(_Key::C),
				_Key::C => None,
			}
		}
	}

	impl GetAsset for _ItemContainer {
		type TKey = _Key;
		type TAsset = _Asset;

		fn get_asset<'a, TAssets>(
			&'a self,
			_: &Self::TKey,
			_: &'a TAssets,
		) -> Option<&'a Self::TAsset>
		where
			TAssets: GetRef<Handle<Self::TAsset>, Self::TAsset>,
		{
			None
		}
	}

	impl ContainsVisibleItemAssets<_View> for _ItemContainer {
		type TVisualizationEntityConstraint = ();

		fn visualization_entity_name(key: &Self::TKey) -> &'static str {
			match key {
				_Key::A => "A",
				_Key::B => "B",
				_Key::C => "C",
			}
		}

		fn visualization_component(_: Option<&Self::TAsset>) -> impl Bundle {}
	}

	#[test]
	fn track_name_if_contained_in_sub_model_names() {
		let mut lookup = Visualize::<_ItemContainer, _View>::default();

		lookup.track(Entity::from_raw(33), &Name::from("A"));

		assert_eq!(
			Visualize {
				entities: HashMap::from([(Name::from("A"), Entity::from_raw(33))]),
				..default()
			},
			lookup,
		);
	}

	#[test]
	fn do_not_track_name_if_not_contained_in_sub_model_names() {
		let mut lookup = Visualize::<_ItemContainer, _View>::default();

		lookup.track(Entity::from_raw(33), &Name::from("D"));

		assert_eq!(
			Visualize {
				entities: HashMap::from([]),
				..default()
			},
			lookup,
		);
	}

	#[test]
	fn is_tracking_true() {
		let mut lookup = Visualize::<_ItemContainer, _View>::default();

		lookup.track(Entity::from_raw(33), &Name::from("A"));

		assert!(lookup.is_tracking(&Entity::from_raw(33)));
	}

	#[test]
	fn is_tracking_false() {
		let mut lookup = Visualize::<_ItemContainer, _View>::default();

		lookup.track(Entity::from_raw(34), &Name::from("A"));

		assert!(!lookup.is_tracking(&Entity::from_raw(33)));
	}

	#[test]
	fn untrack() {
		let mut lookup = Visualize::<_ItemContainer, _View>::default();

		lookup.track(Entity::from_raw(34), &Name::from("A"));
		lookup.track(Entity::from_raw(35), &Name::from("B"));
		lookup.untrack(&Entity::from_raw(34));

		assert_eq!(
			Visualize {
				entities: HashMap::from([(Name::from("B"), Entity::from_raw(35))]),
				..default()
			},
			lookup,
		);
	}

	#[test]
	fn get_entity_by_key() {
		let lookup =
			Visualize::<_ItemContainer, _View>::new([(Name::from("A"), Entity::from_raw(100))]);

		assert_eq!(Some(&Entity::from_raw(100)), lookup.get(&_Key::A));
	}

	#[test]
	fn get_entity_by_other_key() {
		let lookup =
			Visualize::<_ItemContainer, _View>::new([(Name::from("B"), Entity::from_raw(100))]);

		assert_eq!(Some(&Entity::from_raw(100)), lookup.get(&_Key::B));
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn all_view_entities_loaded() {
		let mut app = setup();
		app.world_mut()
			.spawn(Visualize::<_ItemContainer, _View>::new([
				(Name::from("A"), Entity::from_raw(1)),
				(Name::from("B"), Entity::from_raw(2)),
				(Name::from("C"), Entity::from_raw(3)),
			]));

		let loaded = app
			.world_mut()
			.run_system_once(Visualize::<_ItemContainer, _View>::entities_loaded);

		assert_eq!(Loaded(true), loaded);
	}

	#[test]
	fn not_all_view_entities_loaded() {
		let mut app = setup();
		app.world_mut()
			.spawn(Visualize::<_ItemContainer, _View>::new([
				(Name::from("A"), Entity::from_raw(1)),
				(Name::from("C"), Entity::from_raw(3)),
			]));

		let loaded = app
			.world_mut()
			.run_system_once(Visualize::<_ItemContainer, _View>::entities_loaded);

		assert_eq!(Loaded(false), loaded);
	}
}
