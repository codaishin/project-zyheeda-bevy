use common::tools::duration_data::DurationData;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Default, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum LifeTimeDefinition {
	#[default]
	UntilStopped,
	Infinite,
	UntilOutlived(Duration),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum OnActiveLifetime<TDuration> {
	UntilOutlived(TDuration),
	Infinite,
}

impl From<OnActiveLifetime<DurationData>> for OnActiveLifetime<Duration> {
	fn from(value: OnActiveLifetime<DurationData>) -> Self {
		match value {
			OnActiveLifetime::UntilOutlived(data) => Self::UntilOutlived(Duration::from(data)),
			OnActiveLifetime::Infinite => Self::Infinite,
		}
	}
}

impl From<OnActiveLifetime<Duration>> for LifeTimeDefinition {
	fn from(value: OnActiveLifetime<Duration>) -> Self {
		match value {
			OnActiveLifetime::UntilOutlived(duration) => Self::UntilOutlived(duration),
			OnActiveLifetime::Infinite => Self::Infinite,
		}
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum OnAimLifeTime {
	UntilStopped,
	Infinite,
}

impl From<OnAimLifeTime> for LifeTimeDefinition {
	fn from(value: OnAimLifeTime) -> Self {
		match value {
			OnAimLifeTime::UntilStopped => Self::UntilStopped,
			OnAimLifeTime::Infinite => Self::Infinite,
		}
	}
}
