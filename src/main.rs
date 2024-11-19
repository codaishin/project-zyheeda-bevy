use animations::AnimationsPlugin;
use bars::BarsPlugin;
use behaviors::BehaviorsPlugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::CommonPlugin;
use enemy::EnemyPlugin;
use game_state::GameStatePlugin;
use interactions::InteractionsPlugin;
use items::ItemsPlugin;
use light::LightPlugin;
use loading::LoadingPlugin;
use map_generation::MapGenerationPlugin;
use menu::MenuPlugin;
use player::PlayerPlugin;
use prefabs::PrefabsPlugin;
use rendering::RenderingPlugin;
use shaders::ShaderPlugin;
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
	let animations_plugin = AnimationsPlugin;
	let prefabs_plugin = PrefabsPlugin;
	let skills_plugin = SkillsPlugin::depends_on(&animations_plugin);
	let enemy_plugin = EnemyPlugin::depends_on(&prefabs_plugin);
	let map_generation_plugin = MapGenerationPlugin::depends_on(&prefabs_plugin);
	let player_plugin = PlayerPlugin::depends_on(&animations_plugin, &prefabs_plugin);
	let behaviors_plugin = BehaviorsPlugin::depends_on(&animations_plugin, &prefabs_plugin);

	app.add_plugins(DefaultPlugins)
		.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
		.add_plugins(CommonPlugin)
		.add_plugins(prefabs_plugin)
		.add_plugins(ShaderPlugin)
		.add_plugins(InteractionsPlugin)
		.add_plugins(BarsPlugin)
		.add_plugins(ItemsPlugin)
		.add_plugins(animations_plugin)
		.add_plugins(LightPlugin)
		.add_plugins(player_plugin)
		.add_plugins(enemy_plugin)
		.add_plugins(RenderingPlugin)
		.add_plugins(LoadingPlugin)
		.add_plugins(map_generation_plugin)
		.add_plugins(MenuPlugin)
		.add_plugins(skills_plugin)
		.add_plugins(behaviors_plugin)
		.add_plugins(GameStatePlugin)
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
