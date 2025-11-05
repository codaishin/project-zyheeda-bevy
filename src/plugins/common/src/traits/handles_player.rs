use crate::{errors::ErrorData, tools::action_key::slot::SlotKey};
use bevy::{math::InvalidDirectionError, prelude::*};

pub trait HandlesPlayer {
	type TPlayer: Component;
}

pub trait PlayerMainCamera {
	type TPlayerMainCamera: Component + Default;
}

pub trait ConfiguresPlayerSkillAnimations {
	type TAnimationMarker: Component;
	type TError: ErrorData;

	fn start_skill_animation(slot_key: SlotKey) -> Result<Self::TAnimationMarker, Self::TError>;
	fn stop_skill_animation() -> Self::TAnimationMarker;
}

#[derive(Debug, PartialEq)]
pub enum DirectionError<TKey> {
	Invalid(InvalidDirectionError),
	KeyHasNoDirection(TKey),
}
