pub mod animation;
pub mod components;
pub mod events;
pub mod traits;

mod systems;

use animation::MovementAnimations;
use animations::{animation::Animation, components::animation_dispatch::AnimationDispatch};
use bevy::prelude::*;
use common::{
	resources::CamRay,
	states::{GameRunning, MouseContext},
};
use components::{
	cam_orbit::CamOrbit,
	ground_targeted_aoe::{GroundTargetedAoeContact, GroundTargetedAoeProjection},
	projectile::{ProjectileContact, ProjectileProjection},
	shield::{ShieldContact, ShieldProjection},
	Beam,
	Movement,
	MovementConfig,
	PositionBased,
	VelocityBased,
};
use events::MoveInputEvent;
use prefabs::traits::RegisterPrefab;
use systems::{
	attack::{attack, execute_beam::execute_beam},
	chase::chase,
	face::{execute_face::execute_face, get_faces::get_faces},
	idle::idle,
	move_on_orbit::move_on_orbit,
	move_with_target::move_with_target,
	movement::{
		animate_movement::animate_movement,
		execute_move_position_based::execute_move_position_based,
		execute_move_velocity_based::execute_move_velocity_based,
		trigger_event::trigger_move_input_event,
	},
	projectile::{movement::ProjectileMovement, set_position::ProjectileSetPosition},
	shield::position_force_shield,
	update_cool_downs::update_cool_downs,
	update_life_times::update_lifetimes,
};

pub struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<MoveInputEvent>()
			.register_prefab::<Beam>()
			.register_prefab::<ProjectileContact>()
			.register_prefab::<ProjectileProjection>()
			.register_prefab::<ShieldContact>()
			.register_prefab::<ShieldProjection>()
			.register_prefab::<GroundTargetedAoeContact>()
			.register_prefab::<GroundTargetedAoeProjection>()
			.add_systems(
				Update,
				(
					trigger_move_input_event::<CamRay>,
					get_faces.pipe(execute_face::<CamRay>),
				)
					.chain()
					.run_if(in_state(GameRunning::On))
					.run_if(in_state(MouseContext::<KeyCode>::Default)),
			)
			.add_systems(
				Update,
				(move_on_orbit::<CamOrbit>, move_with_target::<CamOrbit>)
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
				),
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
			.add_systems(Update, (chase::<MovementConfig>, attack).chain())
			.add_systems(
				Update,
				(ProjectileContact::set_position, ProjectileContact::movement).chain(),
			)
			.add_systems(Update, GroundTargetedAoeContact::set_position)
			.add_systems(Update, execute_beam)
			.add_systems(Update, position_force_shield);
	}
}
