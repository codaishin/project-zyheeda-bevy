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
pub enum OnActiveLifetime {
	UntilOutlived(Duration),
	Infinite,
}

impl From<OnActiveLifetime> for LifeTimeDefinition {
	fn from(value: OnActiveLifetime) -> Self {
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
