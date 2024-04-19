pub mod components;
mod events;
mod systems;
pub mod traits;

use bevy::{
	app::{App, Plugin, Update},
	ecs::{
		schedule::{common_conditions::in_state, IntoSystemConfigs},
		system::IntoSystem,
	},
	time::Virtual,
};
use common::{components::Player, resources::CamRay, states::GameRunning};
use components::{
	Beam,
	CamOrbit,
	Movement,
	MovementConfig,
	Plasma,
	PositionBased,
	Projectile,
	VelocityBased,
	VoidSphere,
};
use events::MoveInputEvent;
use prefabs::traits::RegisterPrefab;
use systems::{
	attack::{attack, execute_beam::execute_beam},
	chase::chase,
	enemy::enemy,
	face::{execute_face::execute_face, get_faces::get_faces},
	follow::follow,
	idle::idle,
	move_on_orbit::move_on_orbit,
	movement::{
		execute_move_position_based::execute_move_position_based,
		execute_move_velocity_based::execute_move_velocity_based,
		move_player_on_event::move_player_on_event,
		trigger_event::trigger_move_input_event,
	},
	projectile::projectile_behavior,
	update_cool_downs::update_cool_downs,
	update_life_times::update_lifetimes,
};

pub struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<MoveInputEvent>()
			.register_prefab::<Projectile<Plasma>>()
			.register_prefab::<VoidSphere>()
			.register_prefab::<Beam>()
			.add_systems(
				Update,
				(trigger_move_input_event::<CamRay>, move_player_on_event).chain(),
			)
			.add_systems(
				Update,
				(follow::<Player, CamOrbit>, move_on_orbit::<CamOrbit>)
					.run_if(in_state(GameRunning::On)),
			)
			.add_systems(
				Update,
				(update_cool_downs::<Virtual>, update_lifetimes::<Virtual>),
			)
			.add_systems(
				Update,
				(
					execute_move_position_based::<MovementConfig, Movement<PositionBased>, Virtual>
						.pipe(idle),
					execute_move_velocity_based::<MovementConfig, Movement<VelocityBased>>
						.pipe(idle),
					get_faces.pipe(execute_face::<CamRay>),
				)
					.chain(),
			)
			.add_systems(Update, projectile_behavior::<Projectile<Plasma>>)
			.add_systems(Update, (enemy, chase::<MovementConfig>, attack).chain())
			.add_systems(Update, execute_beam);
	}
}
