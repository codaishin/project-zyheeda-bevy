pub mod animation;
pub mod components;
pub mod traits;

mod events;
mod systems;

use animation::MovementAnimations;
use animations::{animation::Animation, components::animation_dispatch::AnimationDispatch};
use bevy::{
	app::{App, Plugin, Update},
	ecs::{schedule::IntoSystemConfigs, system::IntoSystem},
	input::keyboard::KeyCode,
	state::condition::in_state,
	time::Virtual,
	transform::bundles::TransformBundle,
};
use common::{
	components::Player,
	resources::CamRay,
	states::{GameRunning, MouseContext},
};
use components::{
	gravity_well::GravityWell,
	ground_target::GroundTarget,
	projectile::Projectile,
	shield::Shield,
	Beam,
	CamOrbit,
	Movement,
	MovementConfig,
	PositionBased,
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
		animate_movement::animate_movement,
		execute_move_position_based::execute_move_position_based,
		execute_move_velocity_based::execute_move_velocity_based,
		move_player_on_event::move_player_on_event,
		trigger_event::trigger_move_input_event,
	},
	projectile::projectile_behavior,
	replace::replace,
	shield::position_force_shield,
	update_cool_downs::update_cool_downs,
	update_life_times::update_lifetimes,
};

pub struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<MoveInputEvent>()
			.register_prefab::<Projectile>()
			.register_prefab::<VoidSphere>()
			.register_prefab::<Beam>()
			.register_prefab::<Shield>()
			.register_prefab::<GravityWell>()
			.add_systems(
				Update,
				(trigger_move_input_event::<CamRay>, move_player_on_event)
					.chain()
					.run_if(in_state(GameRunning::On))
					.run_if(in_state(MouseContext::<KeyCode>::Default)),
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
			.add_systems(
				Update,
				(
					animate_movement::<
						MovementConfig,
						Movement<PositionBased>,
						Animation,
						MovementAnimations,
						AnimationDispatch,
					>,
					animate_movement::<
						MovementConfig,
						Movement<VelocityBased>,
						Animation,
						MovementAnimations,
						AnimationDispatch,
					>,
				),
			)
			.add_systems(Update, projectile_behavior::<Projectile>)
			.add_systems(Update, (enemy, chase::<MovementConfig>, attack).chain())
			.add_systems(Update, execute_beam)
			.add_systems(Update, position_force_shield)
			.add_systems(Update, replace::<GroundTarget, TransformBundle>);
	}
}
