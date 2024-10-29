use animations::{
	animation::{Animation, PlayMode},
	components::animation_dispatch::AnimationDispatch,
	traits::{GetAnimationPaths, IdleLayer, StartAnimation},
};
use bars::components::Bar;
use behaviors::{
	animation::MovementAnimations,
	components::{MovementConfig, MovementMode},
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier3d::prelude::*;
use common::{
	components::{AssetModel, ColliderRoot, GroundOffset},
	errors::Error,
	systems::init_associated_component::GetAssociated,
	tools::UnitsPerSecond,
	traits::{clamp_zero_positive::ClampZeroPositive, load_asset::Path},
};
use interactions::components::blocker::Blocker;
use light::components::ResponsiveLightTrigger;
use prefabs::traits::{GetOrCreateAssets, Instantiate};

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

impl GetAssociated<AnimationDispatch> for Player {
	fn get_associated_component() -> AnimationDispatch {
		let mut animation_dispatch = AnimationDispatch::default();
		animation_dispatch.start_animation(
			IdleLayer,
			Animation::new(Player::animation_path("Animation1"), PlayMode::Repeat),
		);

		animation_dispatch
	}
}

impl Instantiate for Player {
	fn instantiate(&self, on: &mut EntityCommands, _: impl GetOrCreateAssets) -> Result<(), Error> {
		on.insert((
			Name::from("Player"),
			AssetModel::Path(Player::MODEL_PATH).flip_on(Name::from("metarig")),
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
