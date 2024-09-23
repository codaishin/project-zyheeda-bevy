use animations::{
	animation::{Animation, PlayMode},
	AnimationsPlugin,
};
use bars::{components::Bar, BarsPlugin};
use behaviors::{
	animation::MovementAnimations,
	components::{CamOrbit, MovementConfig, MovementMode, VoidSphere},
	traits::{Orbit, Vec2Radians},
	BehaviorsPlugin,
};
use bevy::{
	core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
	prelude::*,
};
use bevy_rapier3d::prelude::*;
use common::{
	components::{ColliderRoot, GroundOffset, Health, MainCamera, Player},
	states::GameRunning,
	tools::{player_animation_path, UnitsPerSecond},
	traits::clamp_zero_positive::ClampZeroPositive,
	CommonPlugin,
};
use ingame_menu::IngameMenuPlugin;
use interactions::{components::blocker::Blocker, InteractionsPlugin};
use light::{components::ResponsiveLightTrigger, LightPlugin};
use map_generation::MapGenerationPlugin;
use prefabs::PrefabsPlugin;
use project_zyheeda::systems::{
	movement::toggle_walk_run::player_toggle_walk_run,
	void_sphere::ring_rotation::ring_rotation,
};
use shaders::ShaderPlugin;
use skills::SkillsPlugin;
use std::f32::consts::PI;

fn main() -> AppExit {
	let app = &mut App::new();

	prepare_game(app);

	#[cfg(debug_assertions)]
	debug_utils::prepare_debug(app);

	app.run()
}

fn prepare_game(app: &mut App) {
	app.add_plugins(DefaultPlugins)
		.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
		.add_plugins(CommonPlugin)
		.add_plugins(PrefabsPlugin)
		.add_plugins(ShaderPlugin)
		.add_plugins(IngameMenuPlugin)
		.add_plugins(InteractionsPlugin)
		.add_plugins(BarsPlugin)
		.add_plugins(SkillsPlugin)
		.add_plugins(BehaviorsPlugin)
		.add_plugins(AnimationsPlugin)
		.add_plugins(LightPlugin)
		.add_plugins(MapGenerationPlugin)
		.insert_resource(ClearColor(Color::BLACK))
		.add_systems(OnEnter(GameRunning::On), pause_virtual_time::<false>)
		.add_systems(OnExit(GameRunning::On), pause_virtual_time::<true>)
		.add_systems(Startup, setup_simple_3d_scene)
		.add_systems(PreUpdate, player_toggle_walk_run)
		.add_systems(Update, ring_rotation);
}

#[cfg(debug_assertions)]
pub mod debug_utils {
	use super::*;
	use bevy_inspector_egui::quick::WorldInspectorPlugin;
	use interactions::events::{InteractionEvent, Ray};
	use std::ops::Not;

	const FORWARD_GIZMO_COLOR: Color = Color::srgb(0., 0., 1.);

	pub fn prepare_debug(app: &mut App) {
		app.insert_resource(ShowGizmos::No)
			.add_plugins(WorldInspectorPlugin::new())
			.add_plugins(RapierDebugRenderPlugin::default())
			.add_systems(Update, toggle_gizmos)
			.add_systems(
				Update,
				forward_gizmo(&["projectile_spawn", "Player"], &FORWARD_GIZMO_COLOR),
			)
			.add_systems(Update, display_events);
	}

	fn display_events(
		mut collision_events: EventReader<CollisionEvent>,
		mut contact_force_events: EventReader<ContactForceEvent>,
		mut ray_cast_events: EventReader<InteractionEvent<Ray>>,
	) {
		for collision_event in collision_events.read() {
			println!("Received collision event: {:?}", collision_event);
		}

		for contact_force_event in contact_force_events.read() {
			println!("Received contact force event: {:?}", contact_force_event);
		}

		for ray_cast_event in ray_cast_events.read() {
			println!("Received ray cast event: {:?}", ray_cast_event);
		}
	}

	#[derive(Resource, PartialEq, Clone, Copy)]
	enum ShowGizmos {
		Yes,
		No,
	}

	impl Not for ShowGizmos {
		type Output = ShowGizmos;

		fn not(self) -> Self::Output {
			match self {
				ShowGizmos::Yes => ShowGizmos::No,
				ShowGizmos::No => ShowGizmos::Yes,
			}
		}
	}

	fn toggle_gizmos(mut show_gizmos: ResMut<ShowGizmos>, keys: Res<ButtonInput<KeyCode>>) {
		if keys.just_pressed(KeyCode::F11) {
			*show_gizmos = !*show_gizmos;
		}
	}

	fn forward_gizmo<const N: usize>(
		targets: &'static [&str; N],
		color: &'static Color,
	) -> impl Fn(Gizmos, Query<(&Name, &GlobalTransform)>, Res<ShowGizmos>) {
		|mut gizmos, agents, show_gizmos| {
			if *show_gizmos == ShowGizmos::No {
				return;
			}

			for (name, transform) in &agents {
				if targets.contains(&name.as_str()) {
					gizmos.ray(transform.translation(), transform.forward() * 10., *color);
				}
			}
		}
	}
}

fn pause_virtual_time<const PAUSE: bool>(mut time: ResMut<Time<Virtual>>) {
	if PAUSE {
		time.pause();
	} else {
		time.unpause();
	}
}

fn setup_simple_3d_scene(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut next_state: ResMut<NextState<GameRunning>>,
) {
	spawn_player(&mut commands, asset_server);
	spawn_camera(&mut commands);
	spawn_void_spheres(&mut commands);
	next_state.set(GameRunning::On);
}

fn spawn_player(commands: &mut Commands, asset_server: Res<AssetServer>) {
	commands
		.spawn((
			Name::from("Player"),
			Health::new(100.),
			Bar::default(),
			SceneBundle {
				scene: asset_server.load(Player::MODEL_PATH.to_owned() + "#Scene0"),
				..default()
			},
			GroundOffset(Vec3::Y),
			Player,
			Blocker::insert([Blocker::Physical]),
			MovementConfig::Dynamic {
				current_mode: MovementMode::Fast,
				slow_speed: UnitsPerSecond::new(0.75),
				fast_speed: UnitsPerSecond::new(1.5),
			},
			MovementAnimations::new(
				Animation::new(player_animation_path("Animation3"), PlayMode::Repeat),
				Animation::new(player_animation_path("Animation1"), PlayMode::Repeat),
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
}

fn spawn_camera(commands: &mut Commands) {
	let mut transform = Transform::from_translation(Vec3::X);
	let mut orbit = CamOrbit {
		center: Vec3::ZERO,
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
