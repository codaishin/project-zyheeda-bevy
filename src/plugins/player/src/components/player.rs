use super::player_movement::{Config, MovementMode, PlayerMovement};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	attributes::{
		affected_by::{Affected, AffectedBy},
		health::Health,
	},
	components::{
		asset_model::AssetModel,
		collider_relationship::InteractionTarget,
		flip::FlipHorizontally,
		ground_offset::GroundOffset,
		is_blocker::{Blocker, IsBlocker},
		persistent_entity::PersistentEntity,
	},
	effects::{deal_damage::DealDamage, force::Force, gravity::Gravity},
	errors::Error,
	tools::{
		Units,
		UnitsPerSecond,
		action_key::slot::{PlayerSlot, Side, SlotKey},
		animation_key::AnimationKey,
		bone::Bone,
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
		clamp_zero_positive::ClampZeroPositive,
		handles_effect::HandlesEffect,
		handles_lights::HandlesLights,
		handles_skill_behaviors::SkillSpawner,
		iteration::{Iter, IterFinite},
		load_asset::{LoadAsset, Path},
		mapper::Mapper,
		prefab::{Prefab, PrefabEntityCommands},
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Component, SavableComponent, Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(
	PlayerMovement = Player::movement(),
	Transform,
	Visibility,
	Name = "Player",
	AssetModel = Self::MODEL_PATH,
	FlipHorizontally = FlipHorizontally::on("metarig"),
	GroundOffset = Vec3::Y,
	IsBlocker = [Blocker::Character],
	RigidBody = RigidBody::Dynamic,
	InteractionTarget,
	LockedAxes = LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
	GravityScale = GravityScale(0.),
	PersistentEntity,
)]
pub struct Player;

impl Player {
	const MODEL_PATH: &'static str = "models/player.glb";

	fn collider_radius() -> ColliderRadius {
		ColliderRadius(Units::new(0.2))
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
				speed: UnitsPerSecond::new(1.5).into(),
				animation: Animation::new(
					Self::animation_asset(AnimationKey::Run),
					PlayMode::Repeat,
				)
				.into(),
			},
			slow: Config {
				speed: UnitsPerSecond::new(0.75).into(),
				animation: Animation::new(
					Self::animation_asset(AnimationKey::Walk),
					PlayMode::Repeat,
				)
				.into(),
			},
		}
	}
}

impl VisibleSlots for Player {
	fn visible_slots(&self) -> impl Iterator<Item = SlotKey> {
		PlayerSlot::iterator().map(SlotKey::from)
	}
}

impl<'a> Mapper<Bone<'a>, Option<SkillSpawner>> for Player {
	fn map(&self, Bone(bone): Bone) -> Option<SkillSpawner> {
		match bone {
			"skill_spawn" => Some(SkillSpawner::Neutral),
			"skill_spawn_top.R" => Some(SkillSpawner::Slot(PlayerSlot::UPPER_R.into())),
			"skill_spawn_top.L" => Some(SkillSpawner::Slot(PlayerSlot::UPPER_L.into())),
			"skill_spawn_bottom.R" => Some(SkillSpawner::Slot(PlayerSlot::LOWER_R.into())),
			"skill_spawn_bottom.L" => Some(SkillSpawner::Slot(PlayerSlot::LOWER_L.into())),
			_ => None,
		}
	}
}

impl<'a> Mapper<Bone<'a>, Option<EssenceSlot>> for Player {
	fn map(&self, Bone(bone): Bone) -> Option<EssenceSlot> {
		match bone {
			"ArmTopRightData" => Some(EssenceSlot(PlayerSlot::UPPER_R.into())),
			"ArmTopLeftData" => Some(EssenceSlot(PlayerSlot::UPPER_L.into())),
			"ArmBottomRightData" => Some(EssenceSlot(PlayerSlot::LOWER_R.into())),
			"ArmBottomLeftData" => Some(EssenceSlot(PlayerSlot::LOWER_L.into())),
			_ => None,
		}
	}
}

impl<'a> Mapper<Bone<'a>, Option<ForearmSlot>> for Player {
	fn map(&self, Bone(bone): Bone) -> Option<ForearmSlot> {
		match bone {
			"top_forearm.R" => Some(ForearmSlot(PlayerSlot::UPPER_R.into())),
			"top_forearm.L" => Some(ForearmSlot(PlayerSlot::UPPER_L.into())),
			"bottom_forearm.R" => Some(ForearmSlot(PlayerSlot::LOWER_R.into())),
			"bottom_forearm.L" => Some(ForearmSlot(PlayerSlot::LOWER_L.into())),
			_ => None,
		}
	}
}

impl<'a> Mapper<Bone<'a>, Option<HandSlot>> for Player {
	fn map(&self, Bone(bone): Bone) -> Option<HandSlot> {
		match bone {
			"top_hand_slot.R" => Some(HandSlot(PlayerSlot::UPPER_R.into())),
			"top_hand_slot.L" => Some(HandSlot(PlayerSlot::UPPER_L.into())),
			"bottom_hand_slot.R" => Some(HandSlot(PlayerSlot::LOWER_R.into())),
			"bottom_hand_slot.L" => Some(HandSlot(PlayerSlot::LOWER_L.into())),
			_ => None,
		}
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

impl<TInteractions, TLights> Prefab<(TInteractions, TLights)> for Player
where
	TInteractions: HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>
		+ HandlesEffect<Force, TTarget = AffectedBy<Force>>,
	TLights: HandlesLights,
{
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Error> {
		entity
			.try_insert_if_new((
				Health::new(100.).bundle_via::<TInteractions>(),
				Affected::by::<Gravity>().bundle_via::<TInteractions>(),
				Affected::by::<Force>().bundle_via::<TInteractions>(),
			))
			.with_child((
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
