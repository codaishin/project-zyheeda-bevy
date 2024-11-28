use super::Clear;
use bevy::prelude::*;

impl<TComponent> Clear for RemovedComponents<'_, '_, TComponent>
where
	TComponent: Component,
{
	fn clear(&mut self) {
		self.clear();
	}
}
