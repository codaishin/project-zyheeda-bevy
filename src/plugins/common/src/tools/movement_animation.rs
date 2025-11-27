use crate::traits::{accessors::get::Property, handles_animations::Animation};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MovementAnimation(pub Animation);

impl From<Animation> for MovementAnimation {
	fn from(animation: Animation) -> Self {
		MovementAnimation(animation)
	}
}

impl Property for MovementAnimation {
	type TValue<'a> = &'a Animation;
}
