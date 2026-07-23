use crate::resources::standard_materials::StandardMaterials;
use bevy::prelude::*;
use std::collections::{HashSet, hash_map::Entry};

impl StandardMaterials {
	pub(crate) fn track_inserted(
		on_add: On<Insert, MeshMaterial3d<StandardMaterial>>,
		entities: Query<&MeshMaterial3d<StandardMaterial>>,
		mut materials: ResMut<Self>,
	) {
		let Ok(MeshMaterial3d(handle)) = entities.get(on_add.entity) else {
			return;
		};

		match materials.entities.entry(handle.id()) {
			Entry::Occupied(mut entry) => {
				entry.get_mut().insert(on_add.entity);
			}
			Entry::Vacant(entry) => {
				entry.insert(HashSet::from([on_add.entity]));
			}
		};
	}

	pub(crate) fn track_discarded(
		on_discard: On<Discard, MeshMaterial3d<StandardMaterial>>,
		entities: Query<&MeshMaterial3d<StandardMaterial>>,
		mut materials: ResMut<Self>,
	) {
		let Ok(MeshMaterial3d(handle)) = entities.get(on_discard.entity) else {
			return;
		};

		let Entry::Occupied(mut entry) = materials.entities.entry(handle.id()) else {
			return;
		};

		let entities = entry.get_mut();

		entities.remove(&on_discard.entity);
		if !entities.is_empty() {
			return;
		}

		entry.remove();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, new_handle};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<StandardMaterials>();
		app.add_observer(StandardMaterials::track_inserted);
		app.add_observer(StandardMaterials::track_discarded);

		app
	}

	#[test]
	fn track_added() {
		let mut app = setup();
		let handle = new_handle();

		let entity = app.world_mut().spawn(MeshMaterial3d(handle.clone())).id();

		assert_eq!(
			&StandardMaterials {
				entities: HashMap::from([(handle.id(), HashSet::from([entity]))])
			},
			app.world().resource::<StandardMaterials>()
		);
	}

	#[test]
	fn track_added_shared() {
		let mut app = setup();
		let handle = new_handle();

		let entities = [
			app.world_mut().spawn(MeshMaterial3d(handle.clone())).id(),
			app.world_mut().spawn(MeshMaterial3d(handle.clone())).id(),
		];

		assert_eq!(
			&StandardMaterials {
				entities: HashMap::from([(handle.id(), HashSet::from(entities))])
			},
			app.world().resource::<StandardMaterials>()
		);
	}

	#[test]
	fn replace_on_insert() {
		let mut app = setup();
		let handle = new_handle();
		let mut entity = app
			.world_mut()
			.spawn(MeshMaterial3d(new_handle::<StandardMaterial>()));

		entity.insert(MeshMaterial3d(handle.clone()));

		assert_eq!(
			&StandardMaterials {
				entities: HashMap::from([(handle.id(), HashSet::from([entity.id()]))])
			},
			app.world().resource::<StandardMaterials>()
		);
	}

	#[test]
	fn track_partially_removed() {
		let mut app = setup();
		let handle = new_handle();
		let a = app.world_mut().spawn(MeshMaterial3d(handle.clone())).id();
		let b = app.world_mut().spawn(MeshMaterial3d(handle.clone())).id();

		app.world_mut()
			.entity_mut(a)
			.remove::<MeshMaterial3d<StandardMaterial>>();

		assert_eq!(
			&StandardMaterials {
				entities: HashMap::from([(handle.id(), HashSet::from([b]))])
			},
			app.world().resource::<StandardMaterials>()
		);
	}
}
