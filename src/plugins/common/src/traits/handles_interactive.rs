use crate::traits::{
	accessors::get::{TryGetContext, View},
	handles_map_generation::InteractiveType,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use macros::EntityKey;

pub trait HandlesInteractive {
	type TInteractive: SystemParam
		+ for<'c> TryGetContext<Interactive, TContext<'c>: View<InteractiveType>>;
}

#[derive(EntityKey)]
pub struct Interactive {
	pub entity: Entity,
}
