use super::TryRemoveFrom;
use bevy::ecs::{bundle::Bundle, entity::Entity, system::Commands};

impl<'w, 's> TryRemoveFrom for Commands<'w, 's> {
	fn try_remove_from<TBundle: Bundle>(&mut self, entity: Entity) {
		let Some(mut entity) = self.get_entity(entity) else {
			return;
		};
		entity.remove::<TBundle>();
	}
}
