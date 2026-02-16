use bevy::prelude::*;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use testing::ApproxEqual;

/// Use to control an [`Entity`]'s velocity
///
/// Because we cannot implement save-ability for rapier components (we neither own the trait
/// nor the component) we drive all linear velocities through this component.
#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[component(immutable)]
#[savable_component(id = "linear velocity")]
pub(crate) struct LinearVelocity(pub(crate) Vec3);

#[cfg(test)]
impl ApproxEqual<f32> for LinearVelocity {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		self.0.approx_equal(&other.0, tolerance)
	}
}
