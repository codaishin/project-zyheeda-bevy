use crate::{
	errors::ErrorData,
	tools::{
		action_key::{movement::MovementKey, slot::SlotKey},
		collider_radius::ColliderRadius,
		movement_animation::MovementAnimation,
		speed::Speed,
	},
	traits::accessors::get::GetProperty,
};
use bevy::{math::InvalidDirectionError, prelude::*};

pub trait HandlesPlayer {
	type TPlayer: Component;
}

pub trait PlayerMainCamera {
	type TPlayerMainCamera: Component + Default + KeyDirection<TKey = MovementKey>;
}

pub trait ConfiguresPlayerMovement {
	type TPlayerMovement: Component
		+ GetProperty<Speed>
		+ GetProperty<ColliderRadius>
		+ GetProperty<Option<MovementAnimation>>;
}

pub trait ConfiguresPlayerSkillAnimations {
	type TAnimationMarker: Component;
	type TError: ErrorData;

	fn start_skill_animation(slot_key: SlotKey) -> Result<Self::TAnimationMarker, Self::TError>;
	fn stop_skill_animation() -> Self::TAnimationMarker;
}

pub trait KeyDirection {
	type TKey;

	fn key_direction(
		self_transform: &GlobalTransform,
		movement_key: &Self::TKey,
	) -> Result<Dir3, DirectionError<Self::TKey>>;
}

#[derive(Debug, PartialEq)]
pub enum DirectionError<TKey> {
	Invalid(InvalidDirectionError),
	KeyHasNoDirection(TKey),
}
