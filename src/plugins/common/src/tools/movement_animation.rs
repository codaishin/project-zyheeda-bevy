use crate::traits::animation::Animation;

#[derive(Debug, PartialEq, Clone)]
pub struct MovementAnimation(pub Animation);

impl From<Animation> for MovementAnimation {
	fn from(animation: Animation) -> Self {
		MovementAnimation(animation)
	}
}
