use animations::AnimationsPlugin;
use bars::BarsPlugin;
use behaviors::BehaviorsPlugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use camera_control::CameraControlPlugin;
use children_assets_dispatch::ChildrenAssetsDispatchPlugin;
use common::CommonPlugin;
use enemy::EnemyPlugin;
use frame_limiter::FrameLimiterPlugin;
use game_state::GameStatePlugin;
use graphics::GraphicsPlugin;
use interactions::InteractionsPlugin;
use life_cycles::LifeCyclesPlugin;
use light::LightPlugin;
use loading::LoadingPlugin;
use localization::LocalizationPlugin;
use map_generation::MapGenerationPlugin;
use menu::MenuPlugin;
use path_finding::PathFindingPlugin;
use player::PlayerPlugin;
use savegame::SavegamePlugin;
use settings::SettingsPlugin;
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

	let animations = AnimationsPlugin;
	let light = LightPlugin;
	let loading = LoadingPlugin;
	let settings = SettingsPlugin::from_plugin(&loading);
	let localization = LocalizationPlugin::from_plugin(&loading);
	let game_state = GameStatePlugin::from_plugin(&loading);
	let savegame = SavegamePlugin::from_plugin(&settings).with_game_directory(game_dir);
	let life_cycles = LifeCyclesPlugin::from_plugin(&savegame);
	let children_assets_dispatch = ChildrenAssetsDispatchPlugin::from_plugin(&loading);
	let interactions = InteractionsPlugin::from_plugin(&savegame, &life_cycles);
	let enemies = EnemyPlugin::from_plugins(&game_state, &savegame, &interactions);
	let map_generation = MapGenerationPlugin::from_plugin(&light);
	let path_finding = PathFindingPlugin::from_plugin(&map_generation);
	let players = PlayerPlugin::from_plugins(
		&settings,
		&game_state,
		&savegame,
		&animations,
		&interactions,
		&light,
	);
	let behaviors = BehaviorsPlugin::from_plugins(
		&settings,
		&savegame,
		&animations,
		&life_cycles,
		&interactions,
		&path_finding,
		&enemies,
		&players,
	);
	let graphics = GraphicsPlugin::from_plugins(&loading, &interactions, &behaviors);
	let menus = MenuPlugin::from_plugins(&loading, &settings, &localization, &graphics);
	let skills = SkillsPlugin::from_plugins(
		&savegame,
		&life_cycles,
		&interactions,
		&children_assets_dispatch,
		&loading,
		&settings,
		&behaviors,
		&players,
		&menus,
	);
	let bars = BarsPlugin::from_plugins(&life_cycles, &players, &enemies, &graphics);
	let camera_control = CameraControlPlugin::from_plugins(&settings, &players, &graphics);

	app.add_plugins(DefaultPlugins)
		.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
		.add_plugins(CommonPlugin)
		.add_plugins(FrameLimiterPlugin { target_fps: 60 })
		.add_plugins(animations)
		.add_plugins(bars)
		.add_plugins(behaviors)
		.add_plugins(camera_control)
		.add_plugins(children_assets_dispatch)
		.add_plugins(enemies)
		.add_plugins(game_state)
		.add_plugins(graphics)
		.add_plugins(interactions)
		.add_plugins(life_cycles)
		.add_plugins(light)
		.add_plugins(loading)
		.add_plugins(localization)
		.add_plugins(map_generation)
		.add_plugins(menus)
		.add_plugins(path_finding)
		.add_plugins(players)
		.add_plugins(savegame)
		.add_plugins(settings)
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
	use common::tools::action_key::user_input::UserInput;
	use interactions::events::{InteractionEvent, Ray};
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

	fn toggle_gizmos(mut show_gizmos: ResMut<ShowGizmos>, keys: Res<ButtonInput<UserInput>>) {
		if keys.just_pressed(UserInput::from(KeyCode::F11)) {
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
