use agents::AgentsPlugin;
use animations::AnimationsPlugin;
use bars::BarsPlugin;
use behaviors::BehaviorsPlugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use camera_control::CameraControlPlugin;
use common::CommonPlugin;
use frame_limiter::FrameLimiterPlugin;
use graphics::GraphicsPlugin;
use input::InputPlugin;
use light::LightPlugin;
use loading::LoadingPlugin;
use localization::LocalizationPlugin;
use map_generation::MapGenerationPlugin;
use menu::MenuPlugin;
use path_finding::PathFindingPlugin;
use physics::PhysicsPlugin;
use savegame::SavegamePlugin;
use skills::SkillsPlugin;
use std::{
	env::home_dir,
	process::{ExitCode, Termination},
};

fn main() -> ZyheedaAppExit {
	let app = &mut App::new();

	if let Err(error) = prepare_game(app) {
		return ZyheedaAppExit::from(error);
	}

	#[cfg(debug_assertions)]
	debug_utils::prepare_debug(app);

	ZyheedaAppExit::from(app.run())
}

fn prepare_game(app: &mut App) -> Result<(), ZyheedaAppError> {
	let Some(home) = home_dir() else {
		return Err(ZyheedaAppError::NoHomeDirectoryFound);
	};
	let game_dir = home.join("Games").join("Project Zyheeda");

	let loading = LoadingPlugin;
	let input = InputPlugin::from_plugin(&loading);
	let localization = LocalizationPlugin::from_plugin(&loading);
	let savegame = SavegamePlugin::from_plugin(&input).with_game_directory(game_dir);
	let physics = PhysicsPlugin::from_plugin(&savegame);
	let animations = AnimationsPlugin::from_plugin(&savegame);
	let light = LightPlugin::from_plugin(&savegame);
	let map_generation = MapGenerationPlugin::from_plugins(&loading, &savegame, &light);
	let path_finding = PathFindingPlugin::from_plugin(&map_generation);
	let agents = AgentsPlugin::from_plugins(
		&loading,
		&input,
		&savegame,
		&physics,
		&animations,
		&light,
		&map_generation,
	);
	let behaviors = BehaviorsPlugin::from_plugins(
		&input,
		&savegame,
		&animations,
		&physics,
		&path_finding,
		&agents,
	);
	let graphics = GraphicsPlugin::from_plugins(&loading, &savegame, &physics, &behaviors);
	let skills = SkillsPlugin::from_plugins(&savegame, &physics, &loading, &behaviors, &agents);
	let menus = MenuPlugin::from_plugins(
		&loading,
		&savegame,
		&input,
		&localization,
		&graphics,
		&agents,
		&skills,
	);
	let bars = BarsPlugin::from_plugins(&agents, &physics, &graphics);
	let camera_control = CameraControlPlugin::from_plugins(&input, &savegame, &agents, &graphics);

	app.add_plugins(DefaultPlugins)
		.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
		.add_plugins(CommonPlugin)
		.add_plugins(FrameLimiterPlugin { target_fps: 60 })
		.add_plugins(agents)
		.add_plugins(animations)
		.add_plugins(bars)
		.add_plugins(behaviors)
		.add_plugins(camera_control)
		.add_plugins(graphics)
		.add_plugins(physics)
		.add_plugins(light)
		.add_plugins(loading)
		.add_plugins(localization)
		.add_plugins(map_generation)
		.add_plugins(menus)
		.add_plugins(path_finding)
		.add_plugins(savegame)
		.add_plugins(input)
		.add_plugins(skills)
		.insert_resource(ClearColor(Color::BLACK));

	Ok(())
}

enum ZyheedaAppExit {
	AppExit(AppExit),
	Error(ZyheedaAppError),
}

impl From<ZyheedaAppError> for ZyheedaAppExit {
	fn from(error: ZyheedaAppError) -> Self {
		Self::Error(error)
	}
}

impl From<AppExit> for ZyheedaAppExit {
	fn from(app_exit: AppExit) -> Self {
		Self::AppExit(app_exit)
	}
}

impl Termination for ZyheedaAppExit {
	fn report(self) -> std::process::ExitCode {
		match self {
			ZyheedaAppExit::AppExit(app_exit) => app_exit.report(),
			ZyheedaAppExit::Error(error) => {
				error!("{error:?}");
				ExitCode::from(error)
			}
		}
	}
}

#[derive(Debug)]
enum ZyheedaAppError {
	NoHomeDirectoryFound,
}

impl From<ZyheedaAppError> for ExitCode {
	fn from(error: ZyheedaAppError) -> Self {
		match error {
			ZyheedaAppError::NoHomeDirectoryFound => ExitCode::from(10),
		}
	}
}

#[cfg(debug_assertions)]
pub mod debug_utils {
	use super::*;
	use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
	use physics::events::{InteractionEvent, Ray};
	use std::ops::Not;

	const FORWARD_GIZMO_COLOR: Color = Color::srgb(0., 0., 1.);

	pub fn prepare_debug(app: &mut App) {
		app.insert_resource(ShowGizmos::No)
			.add_plugins(EguiPlugin {
				enable_multipass_for_primary_context: true,
			})
			.add_plugins(WorldInspectorPlugin::new())
			.add_plugins(RapierDebugRenderPlugin::default())
			.add_systems(Update, toggle_gizmos)
			.add_systems(
				Update,
				forward_gizmo(&["skill_spawn", "Player"], &FORWARD_GIZMO_COLOR),
			)
			.add_systems(Update, display_events);
	}

	fn display_events(
		mut collision_events: EventReader<CollisionEvent>,
		mut contact_force_events: EventReader<ContactForceEvent>,
		mut ray_cast_events: EventReader<InteractionEvent<Ray>>,
	) {
		for collision_event in collision_events.read() {
			println!("Received collision event: {collision_event:?}");
		}

		for contact_force_event in contact_force_events.read() {
			println!("Received contact force event: {contact_force_event:?}");
		}

		for ray_cast_event in ray_cast_events.read() {
			println!("Received ray cast event: {ray_cast_event:?}");
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
