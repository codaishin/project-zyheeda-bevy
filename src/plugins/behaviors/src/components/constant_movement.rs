use super::VelocityBased;
use bevy::prelude::*;
use common::traits::{accessors::get::GetterRef, handles_behaviors::Speed};
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct ConstantMovement<T> {
	speed: Speed,
	phantom_data: PhantomData<T>,
}

impl ConstantMovement<()> {
	pub(crate) fn velocity_based(speed: Speed) -> ConstantMovement<VelocityBased> {
		ConstantMovement {
			speed,
			phantom_data: PhantomData,
		}
	}
}

impl<T> GetterRef<Speed> for ConstantMovement<T> {
	fn get(&self) -> &Speed {
		let ConstantMovement { speed, .. } = self;

		speed
	}
}
