use bars::components::Bar;
use behaviors::{
	animation::MovementAnimations,
	components::{MovementConfig, MovementMode},
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier3d::prelude::*;
use common::{
	attributes::health::Health,
	blocker::Blocker,
	components::{AssetModel, ColliderRoot, GroundOffset},
	effects::deal_damage::DealDamage,
	errors::Error,
	tools::UnitsPerSecond,
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
		load_asset::Path,
		prefab::{GetOrCreateAssets, Prefab},
	},
};
use light::components::ResponsiveLightTrigger;

#[derive(Component, Default, Debug, PartialEq)]
pub struct Player;

impl Player {
	pub const MODEL_PATH: &'static str = "models/player.glb";

	pub fn animation_path(animation_name: &str) -> Path {
		Path::from(Self::MODEL_PATH.to_owned() + "#" + animation_name)
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

impl<TInteractionsPlugin> Prefab<TInteractionsPlugin> for Player
where
	TInteractionsPlugin: HandlesEffect<DealDamage, TTarget = Health>,
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
				TInteractionsPlugin::attribute(Health::new(100.)),
				Bar::default(),
				GroundOffset(Vec3::Y),
				Blocker::insert([Blocker::Physical]),
				MovementConfig::Dynamic {
					current_mode: MovementMode::Fast,
					slow_speed: UnitsPerSecond::new(0.75),
					fast_speed: UnitsPerSecond::new(1.5),
				},
				MovementAnimations::new(
					Animation::new(Player::animation_path("Animation3"), PlayMode::Repeat),
					Animation::new(Player::animation_path("Animation2"), PlayMode::Repeat),
				),
				RigidBody::Dynamic,
				GravityScale(0.),
				LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
			))
			.with_children(|parent| {
				parent.spawn((
					ResponsiveLightTrigger,
					Collider::capsule(Vec3::new(0.0, 0.2, -0.05), Vec3::new(0.0, 1.4, -0.05), 0.2),
					ColliderRoot(parent.parent_entity()),
				));
			});

		Ok(())
	}
}
