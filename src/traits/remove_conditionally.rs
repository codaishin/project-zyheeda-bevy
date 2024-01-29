use bevy::ecs::{component::Component, system::EntityCommands};

pub trait RemoveConditionally {
	fn remove_conditionally<T: Component>(
		&mut self,
		component: Option<&T>,
		predicate: impl Fn(&T) -> bool,
	);
}

impl RemoveConditionally for EntityCommands<'_, '_, '_> {
	fn remove_conditionally<T: Component>(
		&mut self,
		component: Option<&T>,
		predicate: impl Fn(&T) -> bool,
	) {
		let Some(component) = component else {
			return;
		};
		if !predicate(component) {
			return;
		}
		self.remove::<T>();
	}
}
