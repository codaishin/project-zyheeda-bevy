use animations::{components::Animator, AnimationsPlugin};
use bars::{components::Bar, BarsPlugin};
use behaviors::{
	components::{CamOrbit, MovementConfig, MovementMode},
	traits::{Orbit, Vec2Radians},
	BehaviorsPlugin,
};
use bevy::{
	core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
	prelude::*,
};
use bevy_rapier3d::prelude::*;
use common::{
	components::{Health, Player, VoidSphere},
	tools::UnitsPerSecond,
};
use ingame_menu::IngameMenuPlugin;
use interactions::InteractionsPlugin;
use prefabs::PrefabsPlugin;
use project_zyheeda::systems::{
	movement::toggle_walk_run::player_toggle_walk_run,
	void_sphere::ring_rotation::ring_rotation,
};
use skills::{states::GameRunning, SkillsPlugin};
use std::f32::consts::PI;

fn main() {
	let app = &mut App::new();

	prepare_game(app);

	#[cfg(debug_assertions)]
	debug_utils::prepare_debug(app);

	app.run();
}

fn prepare_game(app: &mut App) {
	app.add_plugins(DefaultPlugins)
		.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
		.add_plugins(PrefabsPlugin)
		.add_plugins(IngameMenuPlugin)
		.add_plugins(InteractionsPlugin)
		.add_plugins(BarsPlugin)
		.add_plugins(SkillsPlugin)
		.add_plugins(BehaviorsPlugin::cam_behavior_if(GameRunning::On))
		.add_plugins(AnimationsPlugin)
		.add_state::<GameRunning>()
		.add_systems(OnEnter(GameRunning::On), pause_virtual_time::<false>)
		.add_systems(OnExit(GameRunning::On), pause_virtual_time::<true>)
		.add_systems(Startup, setup_simple_3d_scene)
		.add_systems(PreUpdate, player_toggle_walk_run)
		.add_systems(Update, ring_rotation);
}

#[cfg(debug_assertions)]
pub mod debug_utils {
	use super::*;
	use bevy::ecs::{archetype::Archetypes, component::Components, entity::Entities};
	use std::ops::Not;

	pub fn prepare_debug(app: &mut App) {
		app.add_plugins(RapierDebugRenderPlugin::default())
			.insert_resource(ShowGizmos::No)
			.add_systems(Update, debug)
			.add_systems(Update, toggle_gizmos)
			.add_systems(
				Update,
				forward_gizmo(&["projectile_spawn", "Player"], &Color::BLUE),
			)
			.add_systems(Update, display_events);
	}

	fn display_events(
		mut collision_events: EventReader<CollisionEvent>,
		mut contact_force_events: EventReader<ContactForceEvent>,
	) {
		for collision_event in collision_events.read() {
			println!("Received collision event: {:?}", collision_event);
		}

		for contact_force_event in contact_force_events.read() {
			println!("Received contact force event: {:?}", contact_force_event);
		}
	}

	fn debug(
		keyboard: Res<Input<KeyCode>>,
		all_entities: Query<Entity>,
		names: Query<&Name>,
		entities: &Entities,
		archetypes: &Archetypes,
		components: &Components,
	) {
		if !keyboard.just_pressed(KeyCode::F12) {
			return;
		}
		for entity in all_entities.iter() {
			println!("Entity: {:?}", entity);
			let name = names.get(entity);
			println!(
				"Entity (Name): {}",
				name.map(|n| n.as_str()).unwrap_or("No Name")
			);
			let Some(entity_location) = entities.get(entity) else {
				return;
			};
			let Some(archetype) = archetypes.get(entity_location.archetype_id) else {
				return;
			};
			for component in archetype.components() {
				if let Some(info) = components.get_info(component) {
					println!("\tComponent: {}", info.name());
				}
			}
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

	fn toggle_gizmos(mut show_gizmos: ResMut<ShowGizmos>, keys: Res<Input<KeyCode>>) {
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
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
	mut next_state: ResMut<NextState<GameRunning>>,
) {
	spawn_plane(&mut commands, &mut meshes, &mut materials);
	spawn_player(&mut commands, asset_server);
	spawn_light(&mut commands);
	spawn_camera(&mut commands);
	spawn_void_spheres(&mut commands);
	next_state.set(GameRunning::On);
}

fn spawn_plane(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
) {
	commands.spawn(PbrBundle {
		mesh: meshes.add(shape::Plane::from_size(5.0).into()),
		material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
		..default()
	});
}

fn spawn_player(commands: &mut Commands, asset_server: Res<AssetServer>) {
	commands.spawn((
		Name::from("Player"),
		Health::new(100),
		Bar::default(),
		SceneBundle {
			scene: asset_server.load("models/player.gltf#Scene0"),
			..default()
		},
		Animator { ..default() },
		Player,
		MovementConfig::Dynamic {
			current_mode: MovementMode::Fast,
			slow_speed: UnitsPerSecond::new(0.75),
			fast_speed: UnitsPerSecond::new(1.5),
		},
	));
}

fn spawn_light(commands: &mut Commands) {
	commands.spawn(PointLightBundle {
		point_light: PointLight {
			intensity: 1500.0,
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(4.0, 8.0, 4.0),
		..default()
	});
}

fn spawn_camera(commands: &mut Commands) {
	let mut transform = Transform::from_translation(Vec3::X);
	let mut orbit = CamOrbit {
		center: Vec3::ZERO,
		distance: 10.,
		sensitivity: 1.,
	};

	orbit.orbit(&mut transform, Vec2Radians::new(-PI / 3., PI / 3.));
	orbit.sensitivity = 0.005;

	commands.spawn((
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
	commands.spawn((
		Name::new("Sphere A"),
		VoidSphere,
		SpatialBundle::from_transform(Transform::from_xyz(2., 0., 2.)),
	));
	commands.spawn((
		Name::new("Sphere B"),
		VoidSphere,
		SpatialBundle::from_transform(Transform::from_xyz(-2., 0., 2.)),
	));
	commands.spawn((
		Name::new("Sphere C"),
		VoidSphere,
		SpatialBundle::from_transform(Transform::from_xyz(2., 0., -2.)),
	));
	commands.spawn((
		Name::new("Sphere D"),
		VoidSphere,
		SpatialBundle::from_transform(Transform::from_xyz(-2., 0., -2.)),
	));
}
