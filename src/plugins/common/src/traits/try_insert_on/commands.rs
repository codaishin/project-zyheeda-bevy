use super::TryInsertOn;
use bevy::ecs::{bundle::Bundle, entity::Entity, system::Commands};

impl<'w, 's> TryInsertOn for Commands<'w, 's> {
	fn try_insert_on<TBundle: Bundle>(&mut self, entity: Entity, bundle: TBundle) {
		let Some(mut entity) = self.get_entity(entity) else {
			return;
		};
		entity.try_insert(bundle);
	}
}
