use super::player_movement::{Config, MovementMode, PlayerMovement};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	attributes::{
		affected_by::{Affected, AffectedBy},
		health::Health,
	},
	blocker::{Blocker, Blockers},
	components::{
		asset_model::AssetModel,
		collider_relationship::InteractionTarget,
		flip::FlipHorizontally,
		ground_offset::GroundOffset,
		persistent_entity::PersistentEntity,
	},
	effects::{deal_damage::DealDamage, force::Force, gravity::Gravity},
	errors::Error,
	tools::{
		Units,
		UnitsPerSecond,
		action_key::slot::{Side, SlotKey},
		animation_key::AnimationKey,
		collider_radius::ColliderRadius,
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
		iteration::{Iter, IterFinite},
		load_asset::Path,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use std::collections::HashMap;

#[derive(Component, Default, Debug, PartialEq)]
#[require(
	PlayerMovement = Player::movement(),
	Transform,
	Visibility,
	Name = "Player",
	AssetModel = Self::MODEL_PATH,
	FlipHorizontally = FlipHorizontally::with(Name::from("metarig")),
	GroundOffset = Vec3::Y,
	Blockers = [Blocker::Character],
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

	fn skill_animation_mask_roots(slot: SlotKey) -> Name {
		match slot {
			SlotKey::TopHand(Side::Left) => Name::from("top_shoulder.L"),
			SlotKey::TopHand(Side::Right) => Name::from("top_shoulder.R"),
			SlotKey::BottomHand(Side::Left) => Name::from("bottom_shoulder.L"),
			SlotKey::BottomHand(Side::Right) => Name::from("bottom_shoulder.R"),
		}
	}

	pub fn animation_asset(animation: AnimationKey<SlotKey>) -> AnimationAsset {
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
			AnimationKey::Other(SlotKey::BottomHand(Side::Left)) => {
				AnimationAsset::Path(Player::animation_path("Animation7"))
			}
			AnimationKey::Other(SlotKey::BottomHand(Side::Right)) => {
				AnimationAsset::Path(Player::animation_path("Animation8"))
			}
			AnimationKey::Other(SlotKey::TopHand(Side::Left)) => {
				AnimationAsset::Path(Player::animation_path("Animation9"))
			}
			AnimationKey::Other(SlotKey::TopHand(Side::Right)) => {
				AnimationAsset::Path(Player::animation_path("Animation10"))
			}
		}
	}

	fn play_animations(animation_key: AnimationKey<SlotKey>) -> (AnimationAsset, AnimationMask) {
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PlayerAnimationMask {
	Body,
	Slot(SlotKey),
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
			PlayerAnimationMask::Body => Some(Self::Slot(SlotKey::iterator().0?)),
			PlayerAnimationMask::Slot(slot_key) => {
				SlotKey::next(&Iter(Some(*slot_key))).map(Self::Slot)
			}
		}
	}
}
impl From<&PlayerAnimationMask> for AnimationMask {
	fn from(mask: &PlayerAnimationMask) -> Self {
		match mask {
			PlayerAnimationMask::Body => 1 << 0,
			PlayerAnimationMask::Slot(SlotKey::TopHand(Side::Left)) => 1 << 1,
			PlayerAnimationMask::Slot(SlotKey::TopHand(Side::Right)) => 1 << 2,
			PlayerAnimationMask::Slot(SlotKey::BottomHand(Side::Left)) => 1 << 3,
			PlayerAnimationMask::Slot(SlotKey::BottomHand(Side::Right)) => 1 << 4,
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
				exclude_roots: SlotKey::iterator()
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
		HashMap::from_iter(AnimationKey::<SlotKey>::iterator().map(Player::play_animations))
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
				.chain(SlotKey::iterator().map(PlayerAnimationMask::Slot))
				.collect::<Vec<_>>(),
			PlayerAnimationMask::iterator()
				.take(10) // prevent infinite loop when broken
				.collect::<Vec<_>>()
		)
	}
}
