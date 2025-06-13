use common::dto::duration_secs_f32::DurationSecsF32;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Default, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum LifeTimeDefinition<TDuration = Duration> {
	#[default]
	UntilStopped,
	Infinite,
	UntilOutlived(TDuration),
}

impl From<LifeTimeDefinition<DurationSecsF32>> for LifeTimeDefinition {
	fn from(lifetime_def_dto: LifeTimeDefinition<DurationSecsF32>) -> Self {
		match lifetime_def_dto {
			LifeTimeDefinition::UntilStopped => Self::UntilStopped,
			LifeTimeDefinition::Infinite => Self::Infinite,
			LifeTimeDefinition::UntilOutlived(dto) => Self::UntilOutlived(Duration::from(dto)),
		}
	}
}

impl From<LifeTimeDefinition> for LifeTimeDefinition<DurationSecsF32> {
	fn from(lifetime_def_duration: LifeTimeDefinition) -> Self {
		match lifetime_def_duration {
			LifeTimeDefinition::UntilStopped => Self::UntilStopped,
			LifeTimeDefinition::Infinite => Self::Infinite,
			LifeTimeDefinition::UntilOutlived(dto) => {
				Self::UntilOutlived(DurationSecsF32::from(dto))
			}
		}
	}
}
