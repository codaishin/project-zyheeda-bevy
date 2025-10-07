use bevy::prelude::*;
use common::traits::handles_agents::{AgentActionTarget, CurrentAction};
use std::collections::HashMap;

#[cfg(test)]
use testing::ApproxEqual;

#[derive(Component, Debug, PartialEq, Default)]
pub struct Actions(pub(crate) HashMap<CurrentAction, AgentActionTarget>);

#[cfg(test)]
impl ApproxEqual<f32> for Actions {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		use AgentActionTarget::{Direction, Entity, Point};

		if self.0.keys().len() != other.0.keys().len() {
			return false;
		}

		for (key, self_value) in self.0.iter() {
			let Some(other_value) = other.0.get(key) else {
				return false;
			};
			match (self_value, other_value) {
				(Point(a), Point(b)) if !a.approx_equal(b, tolerance) => {
					return false;
				}
				(Direction(a), Direction(b)) if !a.approx_equal(b, tolerance) => {
					return false;
				}
				(Entity(a), Entity(b)) if a != b => {
					return false;
				}
				_ => {}
			};
		}

		true
	}
}
