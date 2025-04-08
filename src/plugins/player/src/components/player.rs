use super::player_movement::{Config, MovementMode, PlayerMovement};
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier3d::prelude::*;
use common::{
	attributes::health::Health,
	blocker::{Blocker, BlockerInsertCommand},
	components::{AssetModel, ColliderRoot, GroundOffset, flip::FlipHorizontally},
	effects::deal_damage::DealDamage,
	errors::Error,
	tools::{
		Units,
		UnitsPerSecond,
		animation_key::AnimationKey,
		collider_radius::ColliderRadius,
		slot_key::{Side, SlotKey},
	},
	traits::{
		animation::{
			Animation,
			AnimationMaskDefinition,
			AnimationPriority,
			ConfigureNewAnimationDispatch,
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
		prefab::Prefab,
	},
};
use std::collections::HashMap;

#[derive(Component, Default, Debug, PartialEq)]
#[require(
	PlayerMovement(Player::movement),
	Transform,
	Visibility,
	Name(Self::name),
	AssetModel(Self::model),
	FlipHorizontally(Self::flip_metarig),
	GroundOffset(Self::offset),
	BlockerInsertCommand(Self::blocker),
	RigidBody(Self::rigid_body),
	LockedAxes(Self::locked_axes),
	GravityScale(Self::gravity_scale)
)]
pub struct Player;

impl Player {
	const MODEL_PATH: &'static str = "models/player.glb";

	fn collider_radius() -> ColliderRadius {
		ColliderRadius(Units::new(0.2))
	}

	fn name() -> Name {
		Name::from("Player")
	}

	fn model() -> AssetModel {
		AssetModel::path(Player::MODEL_PATH)
	}

	fn flip_metarig() -> FlipHorizontally {
		FlipHorizontally::with(Name::from("metarig"))
	}

	fn offset() -> GroundOffset {
		GroundOffset(Vec3::Y)
	}

	fn blocker() -> BlockerInsertCommand {
		Blocker::insert([Blocker::Physical])
	}

	fn rigid_body() -> RigidBody {
		RigidBody::Dynamic
	}

	fn locked_axes() -> LockedAxes {
		LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y
	}

	fn gravity_scale() -> GravityScale {
		GravityScale(0.)
	}

	fn anim(animation_name: &str) -> Path {
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

	pub fn animation_paths(animation: AnimationKey<SlotKey>) -> Path {
		match animation {
			AnimationKey::T => Player::anim("Animation0"),
			AnimationKey::Idle => Player::anim("Animation1"),
			AnimationKey::Walk => Player::anim("Animation2"),
			AnimationKey::Run => Player::anim("Animation3"),
			AnimationKey::Other(SlotKey::BottomHand(Side::Left)) => Player::anim("Animation7"),
			AnimationKey::Other(SlotKey::BottomHand(Side::Right)) => Player::anim("Animation8"),
			AnimationKey::Other(SlotKey::TopHand(Side::Left)) => Player::anim("Animation9"),
			AnimationKey::Other(SlotKey::TopHand(Side::Right)) => Player::anim("Animation10"),
		}
	}

	fn play_animations(animation_key: AnimationKey<SlotKey>) -> (Path, AnimationMask) {
		(
			Player::animation_paths(animation_key),
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
				animation: Animation::new(Self::anim("Animation3"), PlayMode::Repeat).into(),
			},
			slow: Config {
				speed: UnitsPerSecond::new(0.75).into(),
				animation: Animation::new(Self::anim("Animation2"), PlayMode::Repeat).into(),
			},
		}
	}

	pub(crate) fn spawn(mut commands: Commands) {
		commands.spawn(Player);
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

	fn animations() -> HashMap<Path, AnimationMask> {
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
			Animation::new(Player::anim("Animation1"), PlayMode::Repeat),
		);
	}
}

impl<TInteractions, TLights> Prefab<(TInteractions, TLights)> for Player
where
	TInteractions: HandlesEffect<DealDamage, TTarget = Health>,
	TLights: HandlesLights,
{
	fn instantiate_on(&self, entity: &mut EntityCommands) -> Result<(), Error> {
		let root = entity.id();
		entity
			.insert(Health::new(100.).bundle_via::<TInteractions>())
			.with_child((
				TLights::responsive_light_trigger(),
				Collider::capsule(
					Vec3::new(0.0, 0.2, -0.05),
					Vec3::new(0.0, 1.4, -0.05),
					**Self::collider_radius(),
				),
				ColliderRoot(root),
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
