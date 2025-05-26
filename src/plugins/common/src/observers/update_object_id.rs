use crate::components::object_id::ObjectId;
use bevy::prelude::*;

impl ObjectId {
	pub(crate) fn update(trigger: Trigger<OnInsert, Self>, mut entities: Query<&mut Self>) {
		let entity = trigger.target();
		let Ok(mut object_id) = entities.get_mut(entity) else {
			return;
		};

		*object_id = object_id.with(entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(ObjectId::update);

		app
	}

	#[test]
	fn update_entity() {
		let mut app = setup();
		let object_id = ObjectId::default();

		let entity = app.world_mut().spawn(object_id);

		assert_eq!(Some(&object_id.with(entity.id())), entity.get::<ObjectId>());
	}

	#[test]
	fn update_entity_when_reinserted() {
		let mut app = setup();
		let object_id = ObjectId::default();

		let mut entity = app.world_mut().spawn(object_id);
		entity.insert(object_id);

		assert_eq!(Some(&object_id.with(entity.id())), entity.get::<ObjectId>());
	}
}
