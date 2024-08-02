use bevy::prelude::{Commands, DetectChanges, Res, Resource};
use common::traits::{iteration::IterFinite, map_value::MapForward};

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct AllKeys<TKey> {
	keys: Vec<TKey>,
}

impl<TKey> AllKeys<TKey> {
	#[cfg(test)]
	pub(crate) fn new(keys: Vec<TKey>) -> Self {
		Self { keys }
	}

	pub(crate) fn keys(&self) -> &Vec<TKey> {
		&self.keys
	}
}

pub(crate) fn collect_all_keys<TEquipmentKey, TMappedKey, TMap>(
	mut commands: Commands,
	map: Res<TMap>,
) where
	TEquipmentKey: IterFinite + Copy + Sync + Send + 'static,
	TMappedKey: Send + Sync + 'static,
	TMap: Resource + MapForward<TEquipmentKey, TMappedKey>,
{
	if !map.is_changed() {
		return;
	}

	commands.insert_resource(AllKeys {
		keys: TEquipmentKey::iterator()
			.map(|key| map.map_forward(key))
			.collect(),
	});
}

#[cfg(test)]
mod tests {
	use std::ops::DerefMut;

	use super::*;
	use bevy::app::{App, Update};
	use common::{test_tools::utils::SingleThreadedApp, traits::iteration::Iter};

	#[derive(Clone, Copy)]
	struct _EquipmentKey;

	impl IterFinite for _EquipmentKey {
		fn iterator() -> Iter<Self> {
			Iter(Some(_EquipmentKey))
		}

		fn next(_: &Iter<Self>) -> Option<Self> {
			None
		}
	}

	#[derive(Debug, PartialEq)]
	struct _MappedKey;

	#[derive(Resource, Default)]
	struct _Map;

	impl MapForward<_EquipmentKey, _MappedKey> for _Map {
		fn map_forward(&self, _: _EquipmentKey) -> _MappedKey {
			_MappedKey
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Map>();
		app.add_systems(Update, collect_all_keys::<_EquipmentKey, _MappedKey, _Map>);

		app
	}

	#[test]
	fn map_keys() {
		let mut app = setup();

		app.update();

		let result = app.world().get_resource::<AllKeys<_MappedKey>>();

		assert_eq!(
			Some(&AllKeys {
				keys: vec![_MappedKey]
			}),
			result
		)
	}

	#[test]
	fn map_nothing_if_not_added() {
		let mut app = setup();

		app.update();

		app.world_mut().remove_resource::<AllKeys<_MappedKey>>();

		app.update();

		let result = app.world().get_resource::<AllKeys<_MappedKey>>();

		assert_eq!(None, result)
	}

	#[test]
	fn map_again_if_map_mutably_accessed() {
		let mut app = setup();

		app.update();

		app.world_mut().remove_resource::<AllKeys<_MappedKey>>();

		app.update();

		app.world_mut().resource_mut::<_Map>().deref_mut();

		app.update();

		let result = app.world().get_resource::<AllKeys<_MappedKey>>();

		assert_eq!(
			Some(&AllKeys {
				keys: vec![_MappedKey]
			}),
			result
		)
	}
}
