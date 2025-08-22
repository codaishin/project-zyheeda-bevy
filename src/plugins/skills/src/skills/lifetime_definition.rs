use common::dto::duration_in_seconds::DurationInSeconds;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Default, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum LifeTimeDefinition<TDuration = Duration> {
	#[default]
	UntilStopped,
	Infinite,
	UntilOutlived(TDuration),
}

impl From<LifeTimeDefinition<DurationInSeconds>> for LifeTimeDefinition {
	fn from(lifetime_def_dto: LifeTimeDefinition<DurationInSeconds>) -> Self {
		match lifetime_def_dto {
			LifeTimeDefinition::UntilStopped => Self::UntilStopped,
			LifeTimeDefinition::Infinite => Self::Infinite,
			LifeTimeDefinition::UntilOutlived(duration) => {
				Self::UntilOutlived(Duration::from(duration))
			}
		}
	}
}

impl From<LifeTimeDefinition> for LifeTimeDefinition<DurationInSeconds> {
	fn from(lifetime_def_duration: LifeTimeDefinition) -> Self {
		match lifetime_def_duration {
			LifeTimeDefinition::UntilStopped => Self::UntilStopped,
			LifeTimeDefinition::Infinite => Self::Infinite,
			LifeTimeDefinition::UntilOutlived(duration) => {
				Self::UntilOutlived(DurationInSeconds::from(duration))
			}
		}
	}
}
