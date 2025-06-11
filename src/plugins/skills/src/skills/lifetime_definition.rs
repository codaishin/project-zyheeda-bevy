use common::dto::duration::DurationDto;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Default, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum LifeTimeDefinition<TDuration = Duration> {
	#[default]
	UntilStopped,
	Infinite,
	UntilOutlived(TDuration),
}

impl From<LifeTimeDefinition<DurationDto>> for LifeTimeDefinition {
	fn from(lifetime_def_dto: LifeTimeDefinition<DurationDto>) -> Self {
		match lifetime_def_dto {
			LifeTimeDefinition::UntilStopped => Self::UntilStopped,
			LifeTimeDefinition::Infinite => Self::Infinite,
			LifeTimeDefinition::UntilOutlived(dto) => Self::UntilOutlived(Duration::from(dto)),
		}
	}
}

impl From<LifeTimeDefinition> for LifeTimeDefinition<DurationDto> {
	fn from(lifetime_def_duration: LifeTimeDefinition) -> Self {
		match lifetime_def_duration {
			LifeTimeDefinition::UntilStopped => Self::UntilStopped,
			LifeTimeDefinition::Infinite => Self::Infinite,
			LifeTimeDefinition::UntilOutlived(dto) => Self::UntilOutlived(DurationDto::from(dto)),
		}
	}
}
