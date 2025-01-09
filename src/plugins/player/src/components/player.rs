use super::player_movement::{Config, MovementMode, PlayerMovement};
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier3d::prelude::*;
use common::{
	attributes::health::Health,
	blocker::Blocker,
	components::{AssetModel, ColliderRoot, GroundOffset},
	effects::deal_damage::DealDamage,
	errors::Error,
	tools::{
		slot_key::{Side, SlotKey},
		UnitsPerSecond,
	},
	traits::{
		animation::{
			Animation,
			AnimationPriority,
			ConfigureNewAnimationDispatch,
			GetAnimationPaths,
			PlayMode,
			StartAnimation,
			StopAnimation,
		},
		clamp_zero_positive::ClampZeroPositive,
		handles_effect::HandlesEffect,
		handles_lights::HandlesLights,
		load_asset::Path,
		prefab::{GetOrCreateAssets, Prefab},
	},
};

#[derive(Component, Default, Debug, PartialEq)]
#[require(PlayerMovement(Player::movement), Transform, Visibility)]
pub struct Player;

impl Player {
	const MODEL_PATH: &'static str = "models/player.glb";

	pub fn animation_path(animation_name: &str) -> Path {
		Path::from(Self::MODEL_PATH.to_owned() + "#" + animation_name)
	}

	pub fn skill_animation(slot: &SlotKey) -> Animation {
		match slot {
			SlotKey::TopHand(Side::Left) => {
				Animation::new(Player::animation_path("Animation6"), PlayMode::Repeat)
			}
			SlotKey::TopHand(Side::Right) => {
				Animation::new(Player::animation_path("Animation7"), PlayMode::Repeat)
			}
			SlotKey::BottomHand(Side::Left) => {
				Animation::new(Player::animation_path("Animation4"), PlayMode::Repeat)
			}
			SlotKey::BottomHand(Side::Right) => {
				Animation::new(Player::animation_path("Animation5"), PlayMode::Repeat)
			}
		}
	}

	fn movement() -> PlayerMovement {
		PlayerMovement {
			mode: MovementMode::Fast,
			fast: Config {
				speed: UnitsPerSecond::new(1.5).into(),
				animation: Animation::new(Self::animation_path("Animation3"), PlayMode::Repeat)
					.into(),
			},
			slow: Config {
				speed: UnitsPerSecond::new(0.75).into(),
				animation: Animation::new(Self::animation_path("Animation2"), PlayMode::Repeat)
					.into(),
			},
		}
	}

	pub(crate) fn spawn(mut commands: Commands) {
		commands.spawn(Player);
	}
}

impl GetAnimationPaths for Player {
	fn animation_paths() -> Vec<Path> {
		vec![
			Player::animation_path("Animation0"),
			Player::animation_path("Animation1"),
			Player::animation_path("Animation2"),
			Player::animation_path("Animation3"),
			Player::animation_path("Animation4"),
			Player::animation_path("Animation5"),
			Player::animation_path("Animation6"),
			Player::animation_path("Animation7"),
		]
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
			Animation::new(Player::animation_path("Animation1"), PlayMode::Repeat),
		);
	}
}

impl<TInteractions, TLights> Prefab<(TInteractions, TLights)> for Player
where
	TInteractions: HandlesEffect<DealDamage, TTarget = Health>,
	TLights: HandlesLights,
{
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		_: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		entity
			.insert((
				Name::from("Player"),
				AssetModel::path(Player::MODEL_PATH).flip_on(Name::from("metarig")),
				Health::new(100.).bundle_via::<TInteractions>(),
				GroundOffset(Vec3::Y),
				Blocker::insert([Blocker::Physical]),
				RigidBody::Dynamic,
				GravityScale(0.),
				LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
			))
			.with_children(|parent| {
				parent.spawn((
					TLights::responsive_light_trigger(),
					Collider::capsule(Vec3::new(0.0, 0.2, -0.05), Vec3::new(0.0, 1.4, -0.05), 0.2),
					ColliderRoot(parent.parent_entity()),
				));
			});

		Ok(())
	}
}
