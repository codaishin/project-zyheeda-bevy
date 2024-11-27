pub mod animation;
pub mod components;
pub mod events;
pub mod traits;

mod systems;

use animation::MovementAnimations;
use bevy::prelude::*;
use common::{
	resources::CamRay,
	states::{game_state::GameState, mouse_context::MouseContext},
	traits::{
		animation::HasAnimationsDispatch,
		handles_lifetime::HandlesLifetime,
		prefab::RegisterPrefab,
		shaders::RegisterForEffectShading,
	},
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
use std::marker::PhantomData;
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
};

pub struct BehaviorsPlugin<TAnimationsPlugin, TPrefabsPlugin, TShadersPlugin, TLifeCyclePlugin>(
	PhantomData<(
		TAnimationsPlugin,
		TPrefabsPlugin,
		TShadersPlugin,
		TLifeCyclePlugin,
	)>,
);

impl<TAnimationsPlugin, TPrefabsPlugin, TShadersPlugin, TLifeCyclePlugin>
	BehaviorsPlugin<TAnimationsPlugin, TPrefabsPlugin, TShadersPlugin, TLifeCyclePlugin>
where
	TAnimationsPlugin: Plugin + HasAnimationsDispatch,
	TPrefabsPlugin: Plugin + RegisterPrefab,
	TShadersPlugin: Plugin + RegisterForEffectShading,
	TLifeCyclePlugin: Plugin + HandlesLifetime,
{
	pub fn depends_on(
		_: &TAnimationsPlugin,
		_: &TPrefabsPlugin,
		_: &TShadersPlugin,
		_: &TLifeCyclePlugin,
	) -> Self {
		Self(
			PhantomData::<(
				TAnimationsPlugin,
				TPrefabsPlugin,
				TShadersPlugin,
				TLifeCyclePlugin,
			)>,
		)
	}
}

impl<TAnimationsPlugin, TPrefabsPlugin, TShadersPlugin, TLifeCyclePlugin> Plugin
	for BehaviorsPlugin<TAnimationsPlugin, TPrefabsPlugin, TShadersPlugin, TLifeCyclePlugin>
where
	TAnimationsPlugin: Plugin + HasAnimationsDispatch,
	TPrefabsPlugin: Plugin + RegisterPrefab,
	TShadersPlugin: Plugin + RegisterForEffectShading,
	TLifeCyclePlugin: Plugin + HandlesLifetime,
{
	fn build(&self, app: &mut App) {
		TPrefabsPlugin::register_prefab::<Beam>(app);
		TPrefabsPlugin::register_prefab::<ProjectileContact>(app);
		TPrefabsPlugin::register_prefab::<ProjectileProjection>(app);
		TPrefabsPlugin::register_prefab::<ShieldContact>(app);
		TPrefabsPlugin::register_prefab::<ShieldProjection>(app);
		TPrefabsPlugin::register_prefab::<GroundTargetedAoeContact>(app);
		TPrefabsPlugin::register_prefab::<GroundTargetedAoeProjection>(app);

		TShadersPlugin::register_for_effect_shading::<ShieldContact>(app);
		TShadersPlugin::register_for_effect_shading::<ShieldProjection>(app);
		TShadersPlugin::register_for_effect_shading::<GroundTargetedAoeContact>(app);

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
						TAnimationsPlugin::TAnimationDispatch,
					>,
					animate_movement::<
						MovementConfig,
						Movement<VelocityBased>,
						MovementAnimations,
						TAnimationsPlugin::TAnimationDispatch,
					>,
				),
			)
			.add_systems(Update, (chase::<MovementConfig>, attack).chain())
			.add_systems(
				Update,
				(ProjectileContact::set_position, ProjectileContact::movement).chain(),
			)
			.add_systems(Update, GroundTargetedAoeContact::set_position)
			.add_systems(Update, execute_beam::<TLifeCyclePlugin::TLifetime>)
			.add_systems(Update, position_force_shield);
	}
}
