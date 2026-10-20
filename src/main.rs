use agents::AgentsPlugin;
use animations::AnimationsPlugin;
use bars::BarsPlugin;
use bevy::prelude::*;
use camera_control::CameraControlPlugin;
use common::CommonPlugin;
use frame_limiter::FrameLimiterPlugin;
use game_states::GameStatesPlugin;
use graphics::GraphicsPlugin;
use input::InputPlugin;
use interactive::InteractivePlugin;
use loading::LoadingPlugin;
use loadout::LoadoutPlugin;
use localization::LocalizationPlugin;
use map_generation::MapGenerationPlugin;
use movement::MovementPlugin;
use path_finding::PathFindingPlugin;
use physics::PhysicsPlugin;
use savegame::SavegamePlugin;
use std::{
	env::home_dir,
	process::{ExitCode, Termination},
};
use ui::UIPlugin;
use zyheeda_core::prelude::*;

fn main() -> ZyheedaAppExit {
	let app = &mut App::new();

	if let Err(error) = prepare_game(app) {
		return ZyheedaAppExit::from(error);
	}

	#[cfg(feature = "debug-utils")]
	debug_utils::prepare_debug(app);

	ZyheedaAppExit::from(app.run())
}

const TARGET_PHYSICS_FPS: PhysicsFPS = physics_fps!(120);
const TARGET_DISPLAY_FPS: DisplayFPS = display_fps!(60);

fn prepare_game(app: &mut App) -> Result<(), ZyheedaAppError> {
	let Some(home) = home_dir() else {
		return Err(ZyheedaAppError::NoHomeDirectoryFound);
	};
	let game_dir = home.join("Games").join("Project Zyheeda");

	let game_states = GameStatesPlugin::default();
	let loading = LoadingPlugin::from_plugins(&game_states);
	let input = InputPlugin::from_plugin(&loading);
	let localization = LocalizationPlugin::from_plugin(&loading);
	let savegame = SavegamePlugin::from_plugin(&game_states).with_game_directory(game_dir);
	let animations = AnimationsPlugin::from_plugin(&savegame);

	#[cfg(not(feature = "debug-utils"))]
	let physics = PhysicsPlugin::new(TARGET_PHYSICS_FPS, &game_states, &savegame, &animations);
	#[cfg(feature = "debug-utils")]
	let physics = PhysicsPlugin::new(
		TARGET_PHYSICS_FPS,
		&game_states,
		&savegame,
		&animations,
		&input,
	);

	let map_generation =
		MapGenerationPlugin::from_plugins(&game_states, &loading, &savegame, &physics);
	let path_finding = PathFindingPlugin::from_plugin(&map_generation);
	let movement = MovementPlugin::from_plugins(
		&game_states,
		&input,
		&savegame,
		&animations,
		&physics,
		&path_finding,
	);

	#[cfg(feature = "debug-utils")]
	let graphics = GraphicsPlugin::new(
		|| bevy_inspector_egui::bevy_egui::PrimaryEguiContext,
		&loading,
		&savegame,
		&physics,
		&map_generation,
	);

	#[cfg(not(feature = "debug-utils"))]
	let graphics = GraphicsPlugin::from_plugins(&loading, &savegame, &physics, &map_generation);

	let loadout =
		LoadoutPlugin::from_plugins(&game_states, &savegame, &physics, &loading, &movement);
	let interactive =
		InteractivePlugin::from_plugin(&loading, &savegame, &physics, &map_generation, &animations);
	let agents = AgentsPlugin::from_plugins(
		&game_states,
		&loading,
		&input,
		&savegame,
		&physics,
		&graphics,
		&interactive,
		&animations,
		&map_generation,
		&movement,
		&loadout,
	);
	let menus = UIPlugin::from_plugins(
		&game_states,
		&loading,
		&savegame,
		&input,
		&localization,
		&graphics,
		&agents,
		&loadout,
	);
	let bars = BarsPlugin::from_plugins(&agents, &physics, &graphics);
	let camera_control = CameraControlPlugin::from_plugins(
		&game_states,
		&input,
		&physics,
		&savegame,
		&agents,
		&graphics,
	);
	let frame_limiter = FrameLimiterPlugin {
		target_fps: TARGET_DISPLAY_FPS,
	};
	let common = CommonPlugin::with_asset_loading(true);

	app.add_plugins(DefaultPlugins)
		.add_plugins(agents)
		.add_plugins(animations)
		.add_plugins(bars)
		.add_plugins(camera_control)
		.add_plugins(common)
		.add_plugins(frame_limiter)
		.add_plugins(game_states)
		.add_plugins(graphics)
		.add_plugins(input)
		.add_plugins(interactive)
		.add_plugins(loading)
		.add_plugins(loadout)
		.add_plugins(localization)
		.add_plugins(map_generation)
		.add_plugins(menus)
		.add_plugins(movement)
		.add_plugins(path_finding)
		.add_plugins(physics)
		.add_plugins(savegame)
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

#[cfg(feature = "debug-utils")]
pub mod debug_utils {
	use super::*;
	use bevy::{
		dev_tools::diagnostics_overlay::{DiagnosticsOverlay, DiagnosticsOverlayPlugin},
		diagnostic::FrameTimeDiagnosticsPlugin,
	};
	use bevy_inspector_egui::{bevy_egui, bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

	pub fn prepare_debug(app: &mut App) {
		app.add_plugins((
			DiagnosticsOverlayPlugin,
			FrameTimeDiagnosticsPlugin::default(),
		))
		.add_systems(PostStartup, |mut commands: Commands| {
			commands.spawn(DiagnosticsOverlay::fps());
		});

		app.insert_resource(bevy_egui::EguiGlobalSettings {
			auto_create_primary_context: false,
			..Default::default()
		})
		.add_plugins(EguiPlugin::default())
		.add_plugins(WorldInspectorPlugin::new());
	}
}
