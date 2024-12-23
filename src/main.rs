use animations::AnimationsPlugin;
use bars::BarsPlugin;
use behaviors::BehaviorsPlugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use children_assets_dispatch::ChildrenAssetsDispatchPlugin;
use common::CommonPlugin;
use enemy::EnemyPlugin;
use frame_limiter::FrameLimiterPlugin;
use game_state::GameStatePlugin;
use interactions::InteractionsPlugin;
use life_cycles::LifeCyclesPlugin;
use light::LightPlugin;
use loading::LoadingPlugin;
use map_generation::MapGenerationPlugin;
use menu::MenuPlugin;
use player::PlayerPlugin;
use prefabs::PrefabsPlugin;
use rendering::RenderingPlugin;
use shaders::ShadersPlugin;
use skills::SkillsPlugin;

fn main() -> AppExit {
	let mut app = App::new();

	let app = &mut app;

	prepare_game(app);

	#[cfg(debug_assertions)]
	debug_utils::prepare_debug(app);

	app.run()
}

fn prepare_game(app: &mut App) {
	let life_cycles_plugin = LifeCyclesPlugin;
	let animations_plugin = AnimationsPlugin;
	let prefabs_plugin = PrefabsPlugin;
	let loading_plugin = LoadingPlugin;
	let game_state_plugin = GameStatePlugin::depends_on(&loading_plugin);
	let light_plugin = LightPlugin::depends_on(&prefabs_plugin);
	let children_assets_dispatch_plugin = ChildrenAssetsDispatchPlugin::depends_on(&loading_plugin);
	let interactions_plugin = InteractionsPlugin::depends_on(&prefabs_plugin, &life_cycles_plugin);
	let bars_plugin = BarsPlugin::depends_on(&life_cycles_plugin);
	let enemy_plugin = EnemyPlugin::depends_on(
		&game_state_plugin,
		&prefabs_plugin,
		&interactions_plugin,
		&bars_plugin,
	);
	let map_generation_plugin = MapGenerationPlugin::depends_on(&prefabs_plugin, &light_plugin);
	let player_plugin = PlayerPlugin::depends_on(
		&game_state_plugin,
		&animations_plugin,
		&prefabs_plugin,
		&interactions_plugin,
		&light_plugin,
		&bars_plugin,
	);
	let behaviors_plugin = BehaviorsPlugin::depends_on(
		&animations_plugin,
		&prefabs_plugin,
		&life_cycles_plugin,
		&interactions_plugin,
		&enemy_plugin,
		&player_plugin,
	);
	let skills_plugin = SkillsPlugin::depends_on(
		&life_cycles_plugin,
		&interactions_plugin,
		&children_assets_dispatch_plugin,
		&loading_plugin,
		&behaviors_plugin,
		&player_plugin,
	);
	let shaders_plugin =
		ShadersPlugin::depends_on(&prefabs_plugin, &interactions_plugin, &behaviors_plugin);
	let rendering_plugin = RenderingPlugin::depends_on(&loading_plugin);
	let menu_plugin = MenuPlugin::depends_on(&loading_plugin);

	app.add_plugins(DefaultPlugins)
		.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
		.add_plugins(CommonPlugin)
		.add_plugins(life_cycles_plugin)
		.add_plugins(prefabs_plugin)
		.add_plugins(shaders_plugin)
		.add_plugins(interactions_plugin)
		.add_plugins(children_assets_dispatch_plugin)
		.add_plugins(bars_plugin)
		.add_plugins(animations_plugin)
		.add_plugins(light_plugin)
		.add_plugins(player_plugin)
		.add_plugins(enemy_plugin)
		.add_plugins(rendering_plugin)
		.add_plugins(loading_plugin)
		.add_plugins(map_generation_plugin)
		.add_plugins(menu_plugin)
		.add_plugins(skills_plugin)
		.add_plugins(behaviors_plugin)
		.add_plugins(game_state_plugin)
		.add_plugins(FrameLimiterPlugin { target_fps: 60 })
		.insert_resource(ClearColor(Color::BLACK));
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
