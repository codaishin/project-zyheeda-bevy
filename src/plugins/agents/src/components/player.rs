use super::player_movement::{Config, MovementMode, PlayerMovement};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	components::{flip::FlipHorizontally, ground_offset::GroundOffset},
	errors::Error,
	tools::{
		Units,
		UnitsPerSecond,
		action_key::slot::{PlayerSlot, Side},
		animation_key::AnimationKey,
		collider_radius::ColliderRadius,
		iter_helpers::{first, next},
	},
	traits::{
		animation::{
			Animation,
			AnimationAsset,
			AnimationMaskDefinition,
			AnimationPriority,
			ConfigureNewAnimationDispatch,
			Directional,
			GetAnimationDefinitions,
			PlayMode,
			StartAnimation,
			StopAnimation,
		},
		handles_agents::AgentType,
		handles_lights::HandlesLights,
		iteration::{Iter, IterFinite},
		load_asset::{LoadAsset, Path},
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use std::collections::HashMap;

#[derive(Component, Default, Debug, PartialEq, Clone)]
#[require(
	PlayerMovement = Player::movement(),
	Name = "Player",
	FlipHorizontally = FlipHorizontally::on("metarig"),
	GroundOffset = Vec3::Y,
	LockedAxes = LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
)]
pub struct Player;

impl Player {
	const MODEL_PATH: &'static str = "models/player.glb";

	fn collider_radius() -> ColliderRadius {
		ColliderRadius(Units::from(0.2))
	}

	fn animation_path(animation_name: &str) -> Path {
		Path::from(Self::MODEL_PATH.to_owned() + "#" + animation_name)
	}

	fn skill_animation_mask_roots(slot: PlayerSlot) -> Name {
		match slot {
			PlayerSlot::Upper(Side::Left) => Name::from("top_shoulder.L"),
			PlayerSlot::Upper(Side::Right) => Name::from("top_shoulder.R"),
			PlayerSlot::Lower(Side::Left) => Name::from("bottom_shoulder.L"),
			PlayerSlot::Lower(Side::Right) => Name::from("bottom_shoulder.R"),
		}
	}

	pub fn animation_asset(animation: AnimationKey<PlayerSlot>) -> AnimationAsset {
		match animation {
			AnimationKey::T => AnimationAsset::Path(Player::animation_path("Animation0")),
			AnimationKey::Idle => AnimationAsset::Path(Player::animation_path("Animation1")),
			AnimationKey::Walk => AnimationAsset::Path(Player::animation_path("Animation2")),
			AnimationKey::Run => AnimationAsset::Directional(Directional {
				forward: Player::animation_path("Animation3"),
				backward: Player::animation_path("Animation4"),
				right: Player::animation_path("Animation5"),
				left: Player::animation_path("Animation6"),
			}),
			AnimationKey::Other(PlayerSlot::Lower(Side::Left)) => {
				AnimationAsset::Path(Player::animation_path("Animation7"))
			}
			AnimationKey::Other(PlayerSlot::Lower(Side::Right)) => {
				AnimationAsset::Path(Player::animation_path("Animation8"))
			}
			AnimationKey::Other(PlayerSlot::Upper(Side::Left)) => {
				AnimationAsset::Path(Player::animation_path("Animation9"))
			}
			AnimationKey::Other(PlayerSlot::Upper(Side::Right)) => {
				AnimationAsset::Path(Player::animation_path("Animation10"))
			}
		}
	}

	fn play_animations(animation_key: AnimationKey<PlayerSlot>) -> (AnimationAsset, AnimationMask) {
		(
			Player::animation_asset(animation_key),
			match animation_key {
				AnimationKey::T => PlayerAnimationMask::all_masks(),
				AnimationKey::Idle => PlayerAnimationMask::all_masks(),
				AnimationKey::Walk => PlayerAnimationMask::all_masks(),
				AnimationKey::Run => PlayerAnimationMask::all_masks(),
				AnimationKey::Other(slot) => AnimationMask::from(PlayerAnimationMask::Slot(slot)),
			},
		)
	}

	fn movement() -> PlayerMovement {
		PlayerMovement {
			mode: MovementMode::Fast,
			collider_radius: Self::collider_radius(),
			fast: Config {
				speed: UnitsPerSecond::from(1.5).into(),
				animation: Animation::new(
					Self::animation_asset(AnimationKey::Run),
					PlayMode::Repeat,
				)
				.into(),
			},
			slow: Config {
				speed: UnitsPerSecond::from(0.75).into(),
				animation: Animation::new(
					Self::animation_asset(AnimationKey::Walk),
					PlayMode::Repeat,
				)
				.into(),
			},
		}
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

impl From<&PlayerAnimationMask> for AnimationMaskDefinition {
	fn from(mask: &PlayerAnimationMask) -> Self {
		match mask {
			PlayerAnimationMask::Body => AnimationMaskDefinition::Mask {
				from_root: Name::from("metarig"),
				exclude_roots: PlayerSlot::iterator()
					.map(Player::skill_animation_mask_roots)
					.collect(),
			},
			PlayerAnimationMask::Slot(slot) => AnimationMaskDefinition::Leaf {
				from_root: Player::skill_animation_mask_roots(*slot),
			},
		}
	}
}

impl From<PlayerAnimationMask> for AnimationMaskDefinition {
	fn from(mask: PlayerAnimationMask) -> Self {
		Self::from(&mask)
	}
}

impl GetAnimationDefinitions for Player {
	type TAnimationMask = PlayerAnimationMask;

	fn animations() -> HashMap<AnimationAsset, AnimationMask> {
		HashMap::from_iter(AnimationKey::<PlayerSlot>::iterator().map(Player::play_animations))
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
				Player::animation_asset(AnimationKey::Idle),
				PlayMode::Repeat,
			),
		);
	}
}

impl<TLights> Prefab<TLights> for Player
where
	TLights: HandlesLights,
{
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Error> {
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
}
