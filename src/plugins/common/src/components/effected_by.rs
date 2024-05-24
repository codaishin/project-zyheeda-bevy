use bevy::ecs::component::Component;
use std::marker::PhantomData;

#[derive(Component)]
pub struct EffectedBy<TEffect> {
	phantom_data: PhantomData<TEffect>,
}

impl<TEffect> Default for EffectedBy<TEffect> {
	fn default() -> Self {
		Self {
			phantom_data: PhantomData,
		}
	}
}
