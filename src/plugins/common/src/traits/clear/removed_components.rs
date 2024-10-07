use super::Clear;
use bevy::prelude::*;

impl<'w, 's, TComponent> Clear for RemovedComponents<'w, 's, TComponent>
where
	TComponent: Component,
{
	fn clear(&mut self) {
		self.clear();
	}
}
