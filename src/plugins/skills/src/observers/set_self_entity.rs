use crate::components::slots::Slots;
use bevy::prelude::*;

impl Slots {
	pub(crate) fn set_self_entity(trigger: Trigger<OnInsert, Self>, mut slots: Query<&mut Self>) {
		let entity = trigger.target();
		let Ok(mut slots) = slots.get_mut(entity) else {
			return;
		};
		slots.self_entity = Some(entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashMap;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(Slots::set_self_entity);

		app
	}

	#[test]
	fn add_entity_on_insert() {
		let mut app = setup();

		let entity = app.world_mut().spawn(Slots::from([]));

		assert_eq!(
			Some(&Slots {
				self_entity: Some(entity.id()),
				items: HashMap::default(),
			}),
			entity.get::<Slots>(),
		);
	}

	#[test]
	fn add_entity_on_reinsert() {
		let mut app = setup();

		let mut entity = app.world_mut().spawn(Slots::from([]));
		entity.insert(Slots::from([]));

		assert_eq!(
			Some(&Slots {
				self_entity: Some(entity.id()),
				items: HashMap::default(),
			}),
			entity.get::<Slots>(),
		);
	}
}
