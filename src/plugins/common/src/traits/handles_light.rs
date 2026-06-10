use crate::{
	tools::Units,
	traits::accessors::get::{TryGetContext, TryGetContextMut},
};
use bevy::prelude::*;
use macros::EntityKey;

pub trait HandlesLight {
	type Lights: for<'c> TryGetContext<TorchLight, TContext<'c>: GetLight>;
	type LightsMut: for<'c> TryGetContextMut<TorchLight, TContext<'c>: SetLight>;
}

#[derive(EntityKey)]
pub struct TorchLight {
	pub entity: Entity,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Light {
	pub intensity: Units,
}

pub trait GetLight {
	fn get_light(&self) -> Light;
}

pub trait SetLight: GetLight {
	fn set_light(&mut self, light: Light);
}
