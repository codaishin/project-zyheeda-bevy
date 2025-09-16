use super::player_movement::{Config, MovementMode, PlayerMovement};
use bevy::{asset::AssetPath, prelude::*};
use bevy_rapier3d::prelude::*;
use common::{
	attributes::{effect_target::EffectTarget, health::Health},
	components::{asset_model::AssetModel, flip::FlipHorizontally, ground_offset::GroundOffset},
	effects::{force::Force, gravity::Gravity},
	errors::Error,
	tools::{
		Units,
		UnitsPerSecond,
		action_key::slot::{PlayerSlot, Side, SlotKey},
		animation_key::AnimationKey,
		attribute::AttributeOnSpawn,
		bone::Bone,
		collider_radius::ColliderRadius,
		iter_helpers::{first, next},
	},
	traits::{
		accessors::get::GetProperty,
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
		handles_lights::HandlesLights,
		handles_skill_behaviors::SkillSpawner,
		iteration::{Iter, IterFinite},
		load_asset::{LoadAsset, Path},
		loadout::LoadoutConfig,
		mapper::Mapper,
		prefab::{Prefab, PrefabEntityCommands},
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use macros::item_asset;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::LazyLock};

#[derive(Component, Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(
	PlayerMovement = Player::movement(),
	Name = "Player",
	AssetModel = Self::MODEL_PATH,
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

static VISIBLE_SLOTS: LazyLock<Vec<SlotKey>> =
	LazyLock::new(|| PlayerSlot::iterator().map(SlotKey::from).collect());

impl VisibleSlots for Player {
	fn visible_slots(&self) -> impl Iterator<Item = SlotKey> {
		VISIBLE_SLOTS.iter().copied()
	}
}

impl<'a> Mapper<Bone<'a>, Option<SkillSpawner>> for Player {
	fn map(&self, Bone(bone): Bone) -> Option<SkillSpawner> {
		match bone {
			"skill_spawn" => Some(SkillSpawner::Neutral),
			"skill_spawn_top.R" => Some(SkillSpawner::Slot(SlotKey::from(PlayerSlot::UPPER_R))),
			"skill_spawn_top.L" => Some(SkillSpawner::Slot(SlotKey::from(PlayerSlot::UPPER_L))),
			"skill_spawn_bottom.R" => Some(SkillSpawner::Slot(SlotKey::from(PlayerSlot::LOWER_R))),
			"skill_spawn_bottom.L" => Some(SkillSpawner::Slot(SlotKey::from(PlayerSlot::LOWER_L))),
			_ => None,
		}
	}
}

impl<'a> Mapper<Bone<'a>, Option<EssenceSlot>> for Player {
	fn map(&self, Bone(bone): Bone) -> Option<EssenceSlot> {
		match bone {
			"ArmTopRightData" => Some(EssenceSlot(SlotKey::from(PlayerSlot::UPPER_R))),
			"ArmTopLeftData" => Some(EssenceSlot(SlotKey::from(PlayerSlot::UPPER_L))),
			"ArmBottomRightData" => Some(EssenceSlot(SlotKey::from(PlayerSlot::LOWER_R))),
			"ArmBottomLeftData" => Some(EssenceSlot(SlotKey::from(PlayerSlot::LOWER_L))),
			_ => None,
		}
	}
}

impl<'a> Mapper<Bone<'a>, Option<ForearmSlot>> for Player {
	fn map(&self, Bone(bone): Bone) -> Option<ForearmSlot> {
		match bone {
			"top_forearm.R" => Some(ForearmSlot(SlotKey::from(PlayerSlot::UPPER_R))),
			"top_forearm.L" => Some(ForearmSlot(SlotKey::from(PlayerSlot::UPPER_L))),
			"bottom_forearm.R" => Some(ForearmSlot(SlotKey::from(PlayerSlot::LOWER_R))),
			"bottom_forearm.L" => Some(ForearmSlot(SlotKey::from(PlayerSlot::LOWER_L))),
			_ => None,
		}
	}
}

impl<'a> Mapper<Bone<'a>, Option<HandSlot>> for Player {
	fn map(&self, Bone(bone): Bone) -> Option<HandSlot> {
		match bone {
			"top_hand_slot.R" => Some(HandSlot(SlotKey::from(PlayerSlot::UPPER_R))),
			"top_hand_slot.L" => Some(HandSlot(SlotKey::from(PlayerSlot::UPPER_L))),
			"bottom_hand_slot.R" => Some(HandSlot(SlotKey::from(PlayerSlot::LOWER_R))),
			"bottom_hand_slot.L" => Some(HandSlot(SlotKey::from(PlayerSlot::LOWER_L))),
			_ => None,
		}
	}
}

impl GetProperty<AttributeOnSpawn<Health>> for Player {
	fn get_property(&self) -> Health {
		Health::new(100.)
	}
}

impl GetProperty<AttributeOnSpawn<EffectTarget<Gravity>>> for Player {
	fn get_property(&self) -> EffectTarget<Gravity> {
		EffectTarget::Affected
	}
}

impl GetProperty<AttributeOnSpawn<EffectTarget<Force>>> for Player {
	fn get_property(&self) -> EffectTarget<Force> {
		EffectTarget::Affected
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

static INVENTORY: LazyLock<Vec<Option<AssetPath<'static>>>> = LazyLock::new(|| {
	vec![
		Some(AssetPath::from(item_asset!("pistol"))),
		Some(AssetPath::from(item_asset!("pistol"))),
		Some(AssetPath::from(item_asset!("pistol"))),
	]
});

static SLOTS: LazyLock<Vec<(SlotKey, Option<AssetPath<'static>>)>> = LazyLock::new(|| {
	vec![
		(
			SlotKey::from(PlayerSlot::Upper(Side::Left)),
			Some(AssetPath::from(item_asset!("pistol"))),
		),
		(
			SlotKey::from(PlayerSlot::Lower(Side::Left)),
			Some(AssetPath::from(item_asset!("pistol"))),
		),
		(
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			Some(AssetPath::from(item_asset!("force_essence"))),
		),
		(
			SlotKey::from(PlayerSlot::Upper(Side::Right)),
			Some(AssetPath::from(item_asset!("force_essence"))),
		),
	]
});

impl LoadoutConfig for Player {
	fn inventory(&self) -> impl Iterator<Item = Option<AssetPath<'static>>> {
		INVENTORY.iter().cloned()
	}

	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<AssetPath<'static>>)> {
		SLOTS.iter().cloned()
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
