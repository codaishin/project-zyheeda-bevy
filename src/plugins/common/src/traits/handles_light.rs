use std::ops::Deref;

use crate::{
	tools::Units,
	traits::accessors::get::{TryGetContext, TryGetContextMut},
};
use bevy::prelude::*;
use macros::EntityKey;

pub trait HandlesLight {
	type TLights: for<'c> TryGetContext<TorchLight, TContext<'c>: GetLight>;
	type TLightsMut: for<'c> TryGetContextMut<TorchLight, TContext<'c>: SetLight>;
}

#[derive(EntityKey)]
pub struct TorchLight {
	pub entity: Entity,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Light {
	pub intensity: Lumen,
}

impl Lumen {
	pub const ZERO: Self = Self(Units::ZERO);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Lumen(pub Units);

impl From<f32> for Lumen {
	fn from(value: f32) -> Self {
		Self(Units::from(value))
	}
}

impl From<i32> for Lumen {
	fn from(value: i32) -> Self {
		Self(Units::from(value as f32))
	}
}

impl Deref for Lumen {
	type Target = f32;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub trait GetLight {
	fn get_light(&self) -> Light;
}

pub trait SetLight: GetLight {
	fn set_light(&mut self, light: Light);
}
