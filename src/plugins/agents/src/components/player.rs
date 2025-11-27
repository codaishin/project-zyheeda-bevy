use super::movement_config::MovementConfig;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	components::{
		collider_relationship::InteractionTarget,
		flip::FlipHorizontally,
		ground_offset::GroundOffset,
	},
	errors::Unreachable,
	tools::{Units, UnitsPerSecond, collider_radius::ColliderRadius},
	traits::{
		animation::AnimationPriority,
		handles_lights::HandlesLights,
		handles_map_generation::AgentType,
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use std::sync::LazyLock;

#[derive(Component, Default, Debug, PartialEq, Clone)]
#[require(
	InteractionTarget,
	MovementConfig = PLAYER_RUN.clone(),
	Name = "Player",
	FlipHorizontally = FlipHorizontally::on("metarig"),
	GroundOffset = Vec3::Y,
	LockedAxes = LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
)]
pub struct Player;

static PLAYER_COLLIDER_RADIUS: LazyLock<Units> = LazyLock::new(|| Units::from(0.2));
pub(crate) static PLAYER_RUN: LazyLock<MovementConfig> = LazyLock::new(|| MovementConfig {
	collider_radius: *PLAYER_COLLIDER_RADIUS,
	speed: UnitsPerSecond::from(1.5),
});
pub(crate) static PLAYER_WALK: LazyLock<MovementConfig> = LazyLock::new(|| MovementConfig {
	collider_radius: *PLAYER_COLLIDER_RADIUS,
	speed: UnitsPerSecond::from(0.75),
});

impl Player {
	fn collider_radius() -> ColliderRadius {
		ColliderRadius(Units::from(0.2))
	}
}

impl From<Player> for AgentType {
	fn from(_: Player) -> Self {
		Self::Player
	}
}

struct Idle;

impl From<Idle> for AnimationPriority {
	fn from(_: Idle) -> Self {
		AnimationPriority::Low
	}
}

impl<TLights> Prefab<TLights> for Player
where
	TLights: HandlesLights,
{
	type TError = Unreachable;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Unreachable> {
		entity.with_child((
			TLights::responsive_light_trigger(),
			Collider::capsule(
				Vec3::new(0.0, 0.2, -0.05),
				Vec3::new(0.0, 1.4, -0.05),
				**Self::collider_radius(),
			),
		));

		Ok(())
	}
}
