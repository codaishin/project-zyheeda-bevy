#[cfg(test)]
mod tests;

use super::{Orbit, Vec2Radians};
use crate::components::CamOrbit;
use bevy::prelude::*;

impl Orbit for CamOrbit {
	fn orbit(&self, agent: &mut Transform, angles: Vec2Radians) {
		let mut arm =
			Transform::from_translation(self.center).looking_at(agent.translation, Vec3::Y);
		let angles = angles * self.sensitivity;

		arm.rotate_y(-angles.x);
		arm.rotate_local_x(angles.y);

		agent.translation = self.center + (arm.forward() * self.distance);
		agent.look_at(self.center, Vec3::Y);
	}
}
