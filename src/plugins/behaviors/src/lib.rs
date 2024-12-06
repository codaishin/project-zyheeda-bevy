pub mod animation;
pub mod components;
pub mod events;
pub mod traits;

mod systems;
mod tools;

use crate::systems::{
	enemy_attack::AttackSystem,
	enemy_behavior::EnemyBehaviorSystem,
	enemy_chase::ChaseSystem,
};
use animation::MovementAnimations;
use bevy::prelude::*;
use common::{
	effects::deal_damage::DealDamage,
	resources::CamRay,
	states::{game_state::GameState, mouse_context::MouseContext},
	traits::{
		animation::HasAnimationsDispatch,
		handles_effect::HandlesEffect,
		handles_effect_shading::HandlesEffectShading,
		handles_enemies::HandlesEnemies,
		handles_interactions::HandlesInteractions,
		handles_player::HandlesPlayer,
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::{
	cam_orbit::CamOrbit,
	ground_targeted_aoe::{GroundTargetedAoeContact, GroundTargetedAoeProjection},
	projectile::{ProjectileContact, ProjectileProjection},
	shield::{ShieldContact, ShieldProjection},
	void_beam::VoidBeam,
	Movement,
	MovementConfig,
	PositionBased,
	VelocityBased,
};
use events::MoveInputEvent;
use std::marker::PhantomData;
use systems::{
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
};
use tools::AttackSpawnerFactory;

pub struct BehaviorsPlugin<TAnimations, TPrefabs, TShaders, TInteractions, TPlayers, TEnemies>(
	PhantomData<(
		TAnimations,
		TPrefabs,
		TShaders,
		TInteractions,
		TPlayers,
		TEnemies,
	)>,
);

impl<TAnimations, TPrefabs, TShaders, TInteractions, TPlayers, TEnemies>
	BehaviorsPlugin<TAnimations, TPrefabs, TShaders, TInteractions, TPlayers, TEnemies>
{
	pub fn depends_on(
		_: &TAnimations,
		_: &TPrefabs,
		_: &TShaders,
		_: &TInteractions,
		_: &TPlayers,
		_: &TEnemies,
	) -> Self {
		Self(
			PhantomData::<(
				TAnimations,
				TPrefabs,
				TShaders,
				TInteractions,
				TPlayers,
				TEnemies,
			)>,
		)
	}
}

impl<TAnimations, TPrefabs, TShaders, TInteractions, TPlayers, TEnemies> Plugin
	for BehaviorsPlugin<TAnimations, TPrefabs, TShaders, TInteractions, TPlayers, TEnemies>
where
	TAnimations: Plugin + HasAnimationsDispatch,
	TPrefabs: Plugin + RegisterPrefab,
	TShaders: Plugin + HandlesEffectShading,
	TInteractions: Plugin + HandlesInteractions + HandlesEffect<DealDamage>,
	TPlayers: Plugin + HandlesPlayer,
	TEnemies: Plugin + HandlesEnemies,
{
	fn build(&self, app: &mut App) {
		TPrefabs::register_prefab::<ProjectileProjection>(app);
		TPrefabs::register_prefab::<GroundTargetedAoeProjection>(app);
		TPrefabs::with_dependency::<TInteractions>()
			.register_prefab::<VoidBeam>(app)
			.register_prefab::<ProjectileContact>(app);
		TPrefabs::with_dependency::<TShaders>()
			.register_prefab::<ShieldContact>(app)
			.register_prefab::<ShieldProjection>(app)
			.register_prefab::<GroundTargetedAoeContact>(app);

		app.add_event::<MoveInputEvent>()
			.add_systems(
				Update,
				(
					trigger_move_input_event::<CamRay>,
					get_faces.pipe(execute_face::<CamRay>),
				)
					.chain()
					.run_if(in_state(GameState::Play))
					.run_if(in_state(MouseContext::<KeyCode>::Default)),
			)
			.add_systems(
				Update,
				(move_on_orbit::<CamOrbit>, move_with_target::<CamOrbit>)
					.run_if(in_state(GameState::Play)),
			)
			.add_systems(Update, update_cool_downs::<Virtual>)
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
						MovementAnimations,
						TAnimations::TAnimationDispatch,
					>,
					animate_movement::<
						MovementConfig,
						Movement<VelocityBased>,
						MovementAnimations,
						TAnimations::TAnimationDispatch,
					>,
				),
			)
			.add_systems(
				Update,
				(
					TEnemies::TEnemy::select_behavior::<TPlayers::TPlayer>,
					TEnemies::TEnemy::chase,
					TEnemies::TEnemy::attack::<AttackSpawnerFactory>,
				)
					.chain(),
			)
			.add_systems(
				Update,
				(ProjectileContact::set_position, ProjectileContact::movement).chain(),
			)
			.add_systems(Update, GroundTargetedAoeContact::set_position)
			.add_systems(Update, position_force_shield);
	}
}
