use crate::{
	dto::duration_in_seconds::DurationInSeconds,
	tools::Units,
	traits::handles_skill_physics::SkillShape,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SphereAoE<TDuration = Duration> {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub lifetime: Option<TDuration>,
	pub max_range: Units,
	pub radius: Units,
}

impl From<SphereAoE> for SkillShape {
	fn from(sphere: SphereAoE) -> Self {
		Self::SphereAoE(sphere)
	}
}

impl From<SphereAoE<DurationInSeconds>> for SphereAoE {
	fn from(with_lifetime_dto: SphereAoE<DurationInSeconds>) -> Self {
		Self {
			lifetime: with_lifetime_dto.lifetime.map(Duration::from),
			max_range: with_lifetime_dto.max_range,
			radius: with_lifetime_dto.radius,
		}
	}
}

impl From<SphereAoE> for SphereAoE<DurationInSeconds> {
	fn from(with_lifetime_duration: SphereAoE) -> Self {
		Self {
			lifetime: with_lifetime_duration.lifetime.map(DurationInSeconds::from),
			max_range: with_lifetime_duration.max_range,
			radius: with_lifetime_duration.radius,
		}
	}
}
