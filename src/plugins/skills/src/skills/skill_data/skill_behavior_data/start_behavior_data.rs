use crate::behaviors::start_behavior::{start_gravity::StartGravity, StartBehavior};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum StartBehaviorData {
	Gravity(StartGravity),
}

impl From<StartBehaviorData> for StartBehavior {
	fn from(value: StartBehaviorData) -> Self {
		match value {
			StartBehaviorData::Gravity(v) => Self::Gravity(v),
		}
	}
}
