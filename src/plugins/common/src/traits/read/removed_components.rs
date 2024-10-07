use super::Read;
use bevy::{ecs::removal_detection::RemovedIter, prelude::*};

impl<'w, 's, 'a, TComponent> Read<'a> for RemovedComponents<'w, 's, TComponent>
where
	TComponent: Component,
{
	type TReturn = RemovedIter<'a>;

	fn read(&'a mut self) -> Self::TReturn {
		self.read()
	}
}
