use super::{accessors::get::RefInto, intersect_at::IntersectAt};
use crate::{
	errors::Error,
	tools::{
		action_key::{movement::MovementKey, slot::SlotKey},
		bone::Bone,
		collider_info::ColliderInfo,
		collider_radius::ColliderRadius,
		movement_animation::MovementAnimation,
		speed::Speed,
	},
	traits::{
		handles_skill_behaviors::SkillSpawner,
		mapper::Mapper,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use bevy::{
	math::{InvalidDirectionError, Ray3d},
	prelude::*,
};

pub trait HandlesPlayer {
	type TPlayer: Component
		+ Default
		+ VisibleSlots
		+ for<'a> Mapper<Bone<'a>, Option<SkillSpawner>>
		+ for<'a> Mapper<Bone<'a>, Option<EssenceSlot>>
		+ for<'a> Mapper<Bone<'a>, Option<HandSlot>>
		+ for<'a> Mapper<Bone<'a>, Option<ForearmSlot>>;
}

pub trait PlayerMainCamera {
	type TPlayerMainCamera: Component + Default + KeyDirection<MovementKey>;
}

pub trait HandlesPlayerCameras {
	type TCamRay: Resource + for<'a> RefInto<'a, Option<&'a Ray3d>> + IntersectAt;
}

pub trait HandlesPlayerMouse {
	type TMouseHover: Resource + for<'a> RefInto<'a, Option<&'a ColliderInfo<Entity>>>;
}

pub trait ConfiguresPlayerMovement {
	type TPlayerMovement: Component
		+ for<'a> RefInto<'a, Speed>
		+ for<'a> RefInto<'a, ColliderRadius>
		+ for<'a> RefInto<'a, Option<&'a MovementAnimation>>;
}

pub trait ConfiguresPlayerSkillAnimations {
	type TAnimationMarker: Component;
	type TError: Into<Error>;

	fn start_skill_animation(slot_key: SlotKey) -> Result<Self::TAnimationMarker, Self::TError>;
	fn stop_skill_animation() -> Self::TAnimationMarker;
}

pub trait KeyDirection<TKey> {
	fn key_direction(
		self_transform: &GlobalTransform,
		movement_key: &TKey,
	) -> Result<Dir3, DirectionError<TKey>>;
}

#[derive(Debug, PartialEq)]
pub enum DirectionError<TKey> {
	Invalid(InvalidDirectionError),
	KeyHasNoDirection(TKey),
}
