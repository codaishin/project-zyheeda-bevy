use crate::traits::update_key_bindings::UpdateKeyBindings;
use bevy::prelude::*;
use common::traits::iterate::Iterate;

impl<T, TKey, TKeyCode> SetKeyBindings<TKey, TKeyCode> for T where
	T: UpdateKeyBindings<TKey, TKeyCode> + Component
{
}

pub(crate) trait SetKeyBindings<TKey, TKeyCode>:
	UpdateKeyBindings<TKey, TKeyCode> + Component + Sized
{
	fn set_key_bindings_from<TKeyMap>(map: Res<TKeyMap>, mut components: Query<&mut Self>)
	where
		TKeyMap: for<'a> Iterate<'a, TItem = (&'a TKey, &'a TKeyCode)> + Resource,
	{
		for mut component in &mut components {
			if !map.is_changed() && !component.is_added() {
				continue;
			}

			component.update_key_bindings(map.as_ref());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{test_tools::utils::SingleThreadedApp, traits::iterate::Iterate};
	use std::collections::{HashMap, hash_map::Iter};

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _Key;

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _KeyCode;

	#[derive(Component, Debug, PartialEq)]
	struct _Component(Vec<(_Key, _KeyCode)>);

	impl<const N: usize> From<[(_Key, _KeyCode); N]> for _Component {
		fn from(keys: [(_Key, _KeyCode); N]) -> Self {
			Self(Vec::from(keys))
		}
	}

	impl UpdateKeyBindings<_Key, _KeyCode> for _Component {
		fn update_key_bindings<TKeyMap>(&mut self, map: &TKeyMap)
		where
			for<'a> TKeyMap: Iterate<'a, TItem = (&'a _Key, &'a _KeyCode)>,
		{
			self.0 = map
				.iterate()
				.map(|(key, key_code)| (*key, *key_code))
				.collect()
		}
	}

	#[derive(Resource)]
	struct _Map(HashMap<_Key, _KeyCode>);

	impl<const N: usize> From<[(_Key, _KeyCode); N]> for _Map {
		fn from(keys: [(_Key, _KeyCode); N]) -> Self {
			Self(HashMap::from(keys))
		}
	}

	impl<'a> Iterate<'a> for _Map {
		type TItem = (&'a _Key, &'a _KeyCode);
		type TIter = Iter<'a, _Key, _KeyCode>;

		fn iterate(&'a self) -> Self::TIter {
			self.0.iter()
		}
	}

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.add_systems(Update, _Component::set_key_bindings_from::<_Map>);

		app
	}

	#[test]
	fn set_key_bindings() {
		let keys = [(_Key, _KeyCode)];
		let map = _Map::from(keys);
		let mut app = setup(map);
		let entity = app.world_mut().spawn(_Component::from([])).id();

		app.update();

		assert_eq!(
			Some(&_Component::from(keys)),
			app.world().entity(entity).get::<_Component>()
		);
	}

	#[test]
	fn set_key_bindings_only_once() {
		let keys = [(_Key, _KeyCode)];
		let map = _Map::from(keys);
		let mut app = setup(map);
		let entity = app.world_mut().spawn(_Component::from([])).id();

		app.update();
		let mut entity_mut = app.world_mut().entity_mut(entity);
		let mut component = entity_mut.get_mut::<_Component>().unwrap();
		*component = _Component::from([]);
		app.update();

		assert_eq!(
			Some(&_Component::from([])),
			app.world().entity(entity).get::<_Component>()
		);
	}

	#[test]
	fn set_key_bindings_again_when_resource_changed() {
		let keys = [(_Key, _KeyCode)];
		let map = _Map::from([]);
		let mut app = setup(map);
		let entity = app.world_mut().spawn(_Component::from([])).id();

		app.update();
		let mut map = app.world_mut().resource_mut::<_Map>();
		*map = _Map::from(keys);
		app.update();

		assert_eq!(
			Some(&_Component::from(keys)),
			app.world().entity(entity).get::<_Component>()
		);
	}

	#[test]
	fn set_key_bindings_again_when_component_added_and_resource_unchanged() {
		let keys = [(_Key, _KeyCode)];
		let map = _Map::from(keys);
		let mut app = setup(map);

		app.update();
		let entity = app.world_mut().spawn(_Component::from([])).id();
		app.update();

		assert_eq!(
			Some(&_Component::from(keys)),
			app.world().entity(entity).get::<_Component>()
		);
	}
}
