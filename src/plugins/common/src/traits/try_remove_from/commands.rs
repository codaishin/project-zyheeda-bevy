use super::TryRemoveFrom;
use bevy::prelude::*;

impl TryRemoveFrom for Commands<'_, '_> {
	fn try_remove_from<TBundle: Bundle>(&mut self, entity: Entity) {
		let Ok(mut entity) = self.get_entity(entity) else {
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
