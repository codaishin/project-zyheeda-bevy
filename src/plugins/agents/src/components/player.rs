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
	tools::{
		Units,
		UnitsPerSecond,
		action_key::slot::{PlayerSlot, Side},
		collider_radius::ColliderRadius,
		iter_helpers::{first, next},
		path::Path,
	},
	traits::{
		animation::{
			AffectedAnimationBones,
			Animation,
			AnimationPath,
			AnimationPriority,
			ConfigureNewAnimationDispatch,
			Directional,
			GetAnimationDefinitions,
			PlayMode,
			StartAnimation,
			StopAnimation,
		},
		handles_lights::HandlesLights,
		handles_map_generation::AgentType,
		iteration::{Iter, IterFinite},
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use std::{collections::HashMap, sync::LazyLock};

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
	const MODEL_PATH: &'static str = "models/player.glb";

	fn collider_radius() -> ColliderRadius {
		ColliderRadius(Units::from(0.2))
	}

	fn animation_path(animation_name: &str) -> Path {
		Path::from(Self::MODEL_PATH.to_owned() + "#" + animation_name)
	}

	fn skill_animation_mask_bone(slot: PlayerSlot) -> Name {
		match slot {
			PlayerSlot::Upper(Side::Left) => Name::from("top_shoulder.L"),
			PlayerSlot::Upper(Side::Right) => Name::from("top_shoulder.R"),
			PlayerSlot::Lower(Side::Left) => Name::from("bottom_shoulder.L"),
			PlayerSlot::Lower(Side::Right) => Name::from("bottom_shoulder.R"),
		}
	}

	pub(crate) fn animation_asset(animation: PlayerAnimationKey) -> AnimationPath {
		match animation {
			PlayerAnimationKey::Idle => AnimationPath::Single(Player::animation_path("Animation1")),
			PlayerAnimationKey::Walk => AnimationPath::Single(Player::animation_path("Animation2")),
			PlayerAnimationKey::Run => AnimationPath::Directional(Directional {
				forward: Player::animation_path("Animation3"),
				backward: Player::animation_path("Animation4"),
				right: Player::animation_path("Animation5"),
				left: Player::animation_path("Animation6"),
			}),
			PlayerAnimationKey::Skill(PlayerSlot::Lower(Side::Left)) => {
				AnimationPath::Single(Player::animation_path("Animation7"))
			}
			PlayerAnimationKey::Skill(PlayerSlot::Lower(Side::Right)) => {
				AnimationPath::Single(Player::animation_path("Animation8"))
			}
			PlayerAnimationKey::Skill(PlayerSlot::Upper(Side::Left)) => {
				AnimationPath::Single(Player::animation_path("Animation9"))
			}
			PlayerAnimationKey::Skill(PlayerSlot::Upper(Side::Right)) => {
				AnimationPath::Single(Player::animation_path("Animation10"))
			}
		}
	}

	fn play_animations(animation_key: PlayerAnimationKey) -> (AnimationPath, AnimationMask) {
		(
			Player::animation_asset(animation_key),
			match animation_key {
				PlayerAnimationKey::Idle => PlayerAnimationMask::all_masks(),
				PlayerAnimationKey::Walk => PlayerAnimationMask::all_masks(),
				PlayerAnimationKey::Run => PlayerAnimationMask::all_masks(),
				PlayerAnimationKey::Skill(slot) => {
					AnimationMask::from(PlayerAnimationMask::Slot(slot))
				}
			},
		)
	}
}

impl From<Player> for AgentType {
	fn from(_: Player) -> Self {
		Self::Player
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PlayerAnimationMask {
	Body,
	Slot(PlayerSlot),
}

impl PlayerAnimationMask {
	fn all_masks() -> AnimationMask {
		PlayerAnimationMask::iterator()
			.map(|mask| AnimationMask::from(&mask))
			.fold(AnimationMask::default(), |a, b| a | b)
	}
}

impl IterFinite for PlayerAnimationMask {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self::Body))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		let Iter(current) = current;
		match current.as_ref()? {
			PlayerAnimationMask::Body => first(PlayerAnimationMask::Slot),
			PlayerAnimationMask::Slot(player_slot) => next(PlayerAnimationMask::Slot, *player_slot),
		}
	}
}

impl From<&PlayerAnimationMask> for AnimationMask {
	fn from(mask: &PlayerAnimationMask) -> Self {
		match mask {
			PlayerAnimationMask::Body => 1 << 0,
			PlayerAnimationMask::Slot(PlayerSlot::Upper(Side::Left)) => 1 << 1,
			PlayerAnimationMask::Slot(PlayerSlot::Upper(Side::Right)) => 1 << 2,
			PlayerAnimationMask::Slot(PlayerSlot::Lower(Side::Left)) => 1 << 3,
			PlayerAnimationMask::Slot(PlayerSlot::Lower(Side::Right)) => 1 << 4,
		}
	}
}

impl From<PlayerAnimationMask> for AnimationMask {
	fn from(mask: PlayerAnimationMask) -> Self {
		Self::from(&mask)
	}
}

impl From<&PlayerAnimationMask> for AffectedAnimationBones {
	fn from(mask: &PlayerAnimationMask) -> Self {
		match mask {
			PlayerAnimationMask::Body => AffectedAnimationBones::SubTree {
				root: Name::from("metarig"),
				until_exclusive: PlayerSlot::iterator()
					.map(Player::skill_animation_mask_bone)
					.collect(),
			},
			PlayerAnimationMask::Slot(slot) => AffectedAnimationBones::Leaf {
				root: Player::skill_animation_mask_bone(*slot),
			},
		}
	}
}

impl From<PlayerAnimationMask> for AffectedAnimationBones {
	fn from(mask: PlayerAnimationMask) -> Self {
		Self::from(&mask)
	}
}

impl GetAnimationDefinitions for Player {
	type TAnimationMask = PlayerAnimationMask;

	fn animations() -> HashMap<AnimationPath, AnimationMask> {
		HashMap::from_iter(PlayerAnimationKey::iterator().map(Player::play_animations))
	}
}

struct Idle;

impl From<Idle> for AnimationPriority {
	fn from(_: Idle) -> Self {
		AnimationPriority::Low
	}
}

impl ConfigureNewAnimationDispatch for Player {
	fn configure_animation_dispatch(
		&self,
		new_animation_dispatch: &mut (impl StartAnimation + StopAnimation),
	) {
		new_animation_dispatch.start_animation(
			Idle,
			Animation::new(
				Player::animation_asset(PlayerAnimationKey::Idle),
				PlayMode::Repeat,
			),
		);
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum PlayerAnimationKey {
	Idle,
	Walk,
	Run,
	Skill(PlayerSlot),
}

impl IterFinite for PlayerAnimationKey {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self::Idle))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match &current.0? {
			Self::Idle => Some(Self::Walk),
			Self::Walk => Some(Self::Run),
			Self::Run => first(Self::Skill),
			Self::Skill(slot) => next(Self::Skill, *slot),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iterate_player_animation_mask() {
		assert_eq!(
			[PlayerAnimationMask::Body]
				.into_iter()
				.chain(PlayerSlot::iterator().map(PlayerAnimationMask::Slot))
				.collect::<Vec<_>>(),
			PlayerAnimationMask::iterator()
				.take(10) // prevent infinite loop when broken
				.collect::<Vec<_>>()
		)
	}

	#[test]
	fn iterate_animation_keys() {
		assert_eq!(
			[
				PlayerAnimationKey::Idle,
				PlayerAnimationKey::Walk,
				PlayerAnimationKey::Run,
			]
			.into_iter()
			.chain(PlayerSlot::iterator().map(PlayerAnimationKey::Skill))
			.collect::<Vec<_>>(),
			PlayerAnimationKey::iterator().take(10).collect::<Vec<_>>()
		)
	}
}
