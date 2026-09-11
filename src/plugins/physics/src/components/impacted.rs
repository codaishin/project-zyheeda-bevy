use bevy::prelude::*;
use common::prelude::*;
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct Impacted {
	pub(crate) impact_points: HashMap<VecNotNan<3>, ImpactStrength>,
}

#[cfg(test)]
impl testing::ApproxEqual<f32> for Impacted {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		if other
			.impact_points
			.keys()
			.any(|k| !self.impact_points.contains_key(k))
		{
			return false;
		}

		for (point, s_strength) in &self.impact_points {
			let Some(o_strength) = other.impact_points.get(point) else {
				return false;
			};

			if !s_strength.approx_equal(o_strength, tolerance) {
				return false;
			}
		}

		true
	}
}
