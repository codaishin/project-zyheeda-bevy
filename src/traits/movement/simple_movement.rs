#[cfg(test)]
mod tests;

use super::{Movement, Seconds};
use crate::components::SimpleMovement;
use bevy::prelude::*;

impl Movement for SimpleMovement {
	fn move_towards(&self, agent: &mut Transform, target: Vec3, delta_time: Seconds) {
		let direction = target - agent.translation;
		let distance = self.speed.unpack() * delta_time;

		match distance < direction.length() {
			true => agent.translation += direction.normalize() * distance,
			false => agent.translation = target,
		};
	}
}
