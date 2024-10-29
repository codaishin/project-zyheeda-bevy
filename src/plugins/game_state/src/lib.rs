mod systems;

use animations::animation::{Animation, PlayMode};
use bars::components::Bar;
use behaviors::{
	animation::MovementAnimations,
	components::{
		cam_orbit::{CamOrbit, CamOrbitCenter},
		MovementConfig,
		MovementMode,
		VoidSphere,
	},
	traits::{Orbit, Vec2Radians},
};
use bevy::{
	core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
	prelude::*,
};
use bevy_rapier3d::prelude::*;
use common::{
	components::{flip::FlipHorizontally, ColliderRoot, GroundOffset, Health, MainCamera},
	states::GameRunning,
	tools::UnitsPerSecond,
	traits::clamp_zero_positive::ClampZeroPositive,
};
use interactions::components::blocker::Blocker;
use light::components::ResponsiveLightTrigger;
use player::components::player::Player;
use std::f32::consts::PI;
use systems::pause_virtual_time::pause_virtual_time;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PostStartup, setup_simple_3d_scene)
			.add_systems(OnEnter(GameRunning::On), pause_virtual_time::<false>)
			.add_systems(OnExit(GameRunning::On), pause_virtual_time::<true>);
	}
}

fn setup_simple_3d_scene(
	mut commands: Commands,
	mut next_state: ResMut<NextState<GameRunning>>,
	asset_server: Res<AssetServer>,
) {
	let player = spawn_player(&mut commands, asset_server);
	spawn_camera(&mut commands, player);
	spawn_void_spheres(&mut commands);
	next_state.set(GameRunning::On);
}

fn spawn_player(commands: &mut Commands, asset_server: Res<AssetServer>) -> Entity {
	commands
		.spawn((
			Name::from("Player"),
			Health::new(100.),
			Bar::default(),
			SceneBundle {
				scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(Player::MODEL_PATH)),
				..default()
			},
			FlipHorizontally::with(Name::from("metarig")),
			GroundOffset(Vec3::Y),
			Player,
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
		})
		.id()
}

fn spawn_camera(commands: &mut Commands, player: Entity) {
	let mut transform = Transform::from_translation(Vec3::X);
	let mut orbit = CamOrbit {
		center: CamOrbitCenter::from(Vec3::ZERO).with_entity(player),
		distance: 15.,
		sensitivity: 1.,
	};

	orbit.orbit(&mut transform, Vec2Radians::new(-PI / 3., PI / 3.));
	orbit.sensitivity = 0.005;

	commands.spawn((
		MainCamera,
		Camera3dBundle {
			camera: Camera {
				hdr: true,
				..default()
			},
			tonemapping: Tonemapping::TonyMcMapface,
			transform,
			..default()
		},
		BloomSettings::default(),
		orbit,
	));
}

fn spawn_void_spheres(commands: &mut Commands) {
	let directions = [
		("Sphere A", Vec3::new(1., 0., 1.)),
		("Sphere B", Vec3::new(-1., 0., 1.)),
		("Sphere C", Vec3::new(1., 0., -1.)),
		("Sphere D", Vec3::new(-1., 0., -1.)),
	];
	let distance = 10.;

	for (name, direction) in directions {
		commands.spawn((
			Name::new(name),
			VoidSphere,
			SpatialBundle::from_transform(Transform::from_translation(direction * distance)),
		));
	}
}
