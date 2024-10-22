use super::TryRemoveFrom;
use bevy::prelude::*;

impl<'w, 's> TryRemoveFrom for Commands<'w, 's> {
	fn try_remove_from<TBundle: Bundle>(&mut self, entity: Entity) {
		let Some(mut entity) = self.get_entity(entity) else {
			return;
		};
		entity.remove::<TBundle>();
	}
}

impl<TCommands> TryRemoveFrom for In<TCommands>
where
	TCommands: TryRemoveFrom,
{
	fn try_remove_from<TBundle: Bundle>(&mut self, entity: Entity) {
		self.0.try_remove_from::<TBundle>(entity);
	}
}
