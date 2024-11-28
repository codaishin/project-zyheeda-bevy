use super::Read;
use bevy::{ecs::removal_detection::RemovedIter, prelude::*};

impl<'a, TComponent> Read<'a> for RemovedComponents<'_, '_, TComponent>
where
	TComponent: Component,
{
	type TReturn = RemovedIter<'a>;

	fn read(&'a mut self) -> Self::TReturn {
		self.read()
	}
}
