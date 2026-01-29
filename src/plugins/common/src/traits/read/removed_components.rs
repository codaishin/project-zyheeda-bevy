use super::Read;
use bevy::{ecs::lifecycle::RemovedIter, prelude::*};

impl<'a, TComponent> Read<'a> for RemovedComponents<'_, '_, TComponent>
where
	TComponent: Component,
{
	type TReturn = RemovedIter<'a>;

	fn read(&'a mut self) -> Self::TReturn {
		self.read()
	}
}
