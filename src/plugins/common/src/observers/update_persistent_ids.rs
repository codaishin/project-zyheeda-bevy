use crate::{components::persistent_id::PersistentId, resources::persistent_ids::PersistentIds};
use bevy::prelude::*;

impl PersistentIds {
	pub(crate) fn update(
		trigger: Trigger<OnInsert, PersistentId>,
		mut persistent_ids: ResMut<PersistentIds>,
		ids: Query<&PersistentId>,
	) {
		let entity = trigger.target();
		let Ok(id) = ids.get(entity) else {
			return;
		};

		persistent_ids.0.insert(*id, entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{test_tools::utils::SingleThreadedApp, traits::accessors::get::Get};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<PersistentIds>();
		app.add_observer(PersistentIds::update);

		app
	}

	#[test]
	fn update_with_entity() {
		let mut app = setup();
		let id = PersistentId::default();

		let entity = app.world_mut().spawn(id).id();

		assert_eq!(
			Some(entity),
			app.world().resource::<PersistentIds>().get(&id)
		);
	}

	#[test]
	fn update_with_entity_when_reinserted() {
		let mut app = setup();
		let id = PersistentId::default();

		let mut entity = app.world_mut().spawn(PersistentId::default());
		entity.insert(id);

		let entity = entity.id();
		assert_eq!(
			Some(entity),
			app.world().resource::<PersistentIds>().get(&id)
		);
	}
}
