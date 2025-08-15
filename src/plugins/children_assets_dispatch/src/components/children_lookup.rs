use bevy::prelude::{Component, Entity, Name, Query};
use common::traits::{
	accessors::get::GetRef,
	handles_assets_for_children::{ChildAssetDefinition, ChildName},
	handles_load_tracking::Loaded,
	iteration::IterFinite,
	track::{IsTracking, Track, Untrack},
};
use std::{collections::HashMap, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct ChildrenLookup<TParent, TMarker> {
	pub(crate) entities: HashMap<Name, Entity>,
	phantom_data: PhantomData<(TParent, TMarker)>,
}

impl<TParent, TMarker> ChildrenLookup<TParent, TMarker>
where
	TParent: ChildAssetDefinition<TMarker> + Sync + Send + 'static,
	TParent::TChildKey: IterFinite,
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
		let key_count = TParent::TChildKey::iterator().count();
		Loaded(
			visualizers
				.iter()
				.all(|visualizer| visualizer.entities.len() == key_count),
		)
	}
}

impl<TParent, TMarker> Default for ChildrenLookup<TParent, TMarker> {
	fn default() -> Self {
		Self {
			entities: HashMap::new(),
			phantom_data: PhantomData,
		}
	}
}

impl<TParent, TMarker> Track<Name> for ChildrenLookup<TParent, TMarker>
where
	TParent: ChildAssetDefinition<TMarker>,
	TParent::TChildKey: IterFinite,
{
	fn track(&mut self, entity: Entity, name: &Name) {
		let entity_keys = TParent::TChildKey::iterator();
		let entity_not_valid = !entity_keys
			.map(|key| key.child_name())
			.any(|entity_name| entity_name == name.as_str());

		if entity_not_valid {
			return;
		}

		self.entities.insert(name.clone(), entity);
	}
}

impl<TContainer, TMarker> IsTracking<Name> for ChildrenLookup<TContainer, TMarker> {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.entities.values().any(|e| e == entity)
	}
}

impl<TContainer, TMarker> Untrack<Name> for ChildrenLookup<TContainer, TMarker> {
	fn untrack(&mut self, entity: &Entity) {
		self.entities.retain(|_, e| e != entity);
	}
}

impl<TContainer, TMarker> GetRef<TContainer::TChildKey> for ChildrenLookup<TContainer, TMarker>
where
	TContainer: ChildAssetDefinition<TMarker>,
{
	type TValue<'a>
		= &'a Entity
	where
		Self: 'a;

	fn get_ref(&self, key: &TContainer::TChildKey) -> Option<&Entity> {
		self.entities.get(&Name::from(key.child_name()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{
		get_asset::GetAsset,
		handles_assets_for_children::{ChildAssetComponent, ChildName},
		iteration::Iter,
	};

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
			TAssets: GetRef<Handle<Self::TAsset>, TValue<'a> = &'a Self::TAsset>,
		{
			None
		}
	}

	impl ChildName<_View> for _Key {
		fn child_name(&self) -> &'static str {
			match self {
				_Key::A => "A",
				_Key::B => "B",
				_Key::C => "C",
			}
		}
	}

	impl ChildAssetComponent<_View> for _Asset {
		type TComponent = _ChildAssetComponent;

		fn component(_: Option<&Self>) -> Self::TComponent {
			_ChildAssetComponent
		}
	}

	impl ChildAssetDefinition<_View> for _ItemContainer {
		type TChildKey = _Key;
		type TChildFilter = ();

		type TChildAsset = _Asset;
	}

	#[derive(Component)]
	struct _ChildAssetComponent;

	#[test]
	fn track_name_if_contained_in_sub_model_names() {
		let mut lookup = ChildrenLookup::<_ItemContainer, _View>::default();

		lookup.track(Entity::from_raw(33), &Name::from("A"));

		assert_eq!(
			ChildrenLookup {
				entities: HashMap::from([(Name::from("A"), Entity::from_raw(33))]),
				..default()
			},
			lookup,
		);
	}

	#[test]
	fn do_not_track_name_if_not_contained_in_sub_model_names() {
		let mut lookup = ChildrenLookup::<_ItemContainer, _View>::default();

		lookup.track(Entity::from_raw(33), &Name::from("D"));

		assert_eq!(
			ChildrenLookup {
				entities: HashMap::from([]),
				..default()
			},
			lookup,
		);
	}

	#[test]
	fn is_tracking_true() {
		let mut lookup = ChildrenLookup::<_ItemContainer, _View>::default();

		lookup.track(Entity::from_raw(33), &Name::from("A"));

		assert!(lookup.is_tracking(&Entity::from_raw(33)));
	}

	#[test]
	fn is_tracking_false() {
		let mut lookup = ChildrenLookup::<_ItemContainer, _View>::default();

		lookup.track(Entity::from_raw(34), &Name::from("A"));

		assert!(!lookup.is_tracking(&Entity::from_raw(33)));
	}

	#[test]
	fn untrack() {
		let mut lookup = ChildrenLookup::<_ItemContainer, _View>::default();

		lookup.track(Entity::from_raw(34), &Name::from("A"));
		lookup.track(Entity::from_raw(35), &Name::from("B"));
		lookup.untrack(&Entity::from_raw(34));

		assert_eq!(
			ChildrenLookup {
				entities: HashMap::from([(Name::from("B"), Entity::from_raw(35))]),
				..default()
			},
			lookup,
		);
	}

	#[test]
	fn get_entity_by_key() {
		let lookup = ChildrenLookup::<_ItemContainer, _View>::new([(
			Name::from("A"),
			Entity::from_raw(100),
		)]);

		assert_eq!(Some(&Entity::from_raw(100)), lookup.get_ref(&_Key::A));
	}

	#[test]
	fn get_entity_by_other_key() {
		let lookup = ChildrenLookup::<_ItemContainer, _View>::new([(
			Name::from("B"),
			Entity::from_raw(100),
		)]);

		assert_eq!(Some(&Entity::from_raw(100)), lookup.get_ref(&_Key::B));
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn all_view_entities_loaded() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.spawn(ChildrenLookup::<_ItemContainer, _View>::new([
				(Name::from("A"), Entity::from_raw(1)),
				(Name::from("B"), Entity::from_raw(2)),
				(Name::from("C"), Entity::from_raw(3)),
			]));

		let loaded = app
			.world_mut()
			.run_system_once(ChildrenLookup::<_ItemContainer, _View>::entities_loaded)?;

		assert_eq!(Loaded(true), loaded);
		Ok(())
	}

	#[test]
	fn not_all_view_entities_loaded() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.spawn(ChildrenLookup::<_ItemContainer, _View>::new([
				(Name::from("A"), Entity::from_raw(1)),
				(Name::from("C"), Entity::from_raw(3)),
			]));

		let loaded = app
			.world_mut()
			.run_system_once(ChildrenLookup::<_ItemContainer, _View>::entities_loaded)?;

		assert_eq!(Loaded(false), loaded);
		Ok(())
	}
}
