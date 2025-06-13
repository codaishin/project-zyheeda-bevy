use crate::traits::handles_saving::SavableComponent;
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

impl SavableComponent for Transform {
	type TDto = Self;
}

impl SavableComponent for Name {
	type TDto = Self;
}

impl SavableComponent for Velocity {
	type TDto = Self;
}
