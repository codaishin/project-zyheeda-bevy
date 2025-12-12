use super::movement_config::MovementConfig;
use bevy::prelude::*;
use common::{
	components::{
		collider_relationship::InteractionTarget,
		flip::FlipHorizontally,
		ground_offset::GroundOffset,
		is_blocker::Blocker,
	},
	errors::Unreachable,
	tools::{Units, UnitsPerSecond},
	traits::{
		handles_animations::AnimationPriority,
		handles_map_generation::AgentType,
		handles_physics::colliders::{Collider, ColliderType, HandlesColliders, Shape},
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
	GroundOffset = GROUND_OFFSET,
)]
pub struct Player;

static GROUND_OFFSET: Vec3 = Vec3::new(0., 0.7, 0.);
static PLAYER_COLLIDER_RADIUS: LazyLock<Units> = LazyLock::new(|| Units::from(0.2));
static PLAYER_COLLIDER_HEIGHT: LazyLock<Units> = LazyLock::new(|| Units::from(0.4));
pub(crate) static PLAYER_RUN: LazyLock<MovementConfig> = LazyLock::new(|| MovementConfig {
	collider_radius: *PLAYER_COLLIDER_RADIUS,
	speed: UnitsPerSecond::from(1.5),
});
pub(crate) static PLAYER_WALK: LazyLock<MovementConfig> = LazyLock::new(|| MovementConfig {
	collider_radius: *PLAYER_COLLIDER_RADIUS,
	speed: UnitsPerSecond::from(0.75),
});

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

impl<TPhysics> Prefab<TPhysics> for Player
where
	TPhysics: HandlesColliders,
{
	type TError = Unreachable;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Unreachable> {
		let shape = Shape::Capsule {
			half_y: **PLAYER_COLLIDER_HEIGHT,
			radius: **PLAYER_COLLIDER_RADIUS,
		};
		let collider = Collider::from_shape(shape)
			.with_center_offset(GROUND_OFFSET)
			.with_collider_type(ColliderType::Agent)
			.with_blocker_types([Blocker::Character]);

		entity.try_insert_if_new(TPhysics::TCollider::from(collider));

		Ok(())
	}
}
