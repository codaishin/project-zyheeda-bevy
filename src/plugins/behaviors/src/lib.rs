pub mod animation;
pub mod components;
pub mod events;
pub mod traits;

mod systems;

use animation::MovementAnimations;
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	effects::deal_damage::DealDamage,
	resources::CamRay,
	states::{game_state::GameState, mouse_context::MouseContext},
	traits::{
		animation::HasAnimationsDispatch,
		handles_destruction::HandlesDestruction,
		handles_effect::HandlesEffect,
		handles_effect_shading::HandlesEffectShading,
		handles_interactions::HandlesInteractions,
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::{
	cam_orbit::CamOrbit,
	ground_target::GroundTarget,
	set_position_and_rotation::SetPositionAndRotation,
	set_to_move_forward::SetVelocityForward,
	skill_behavior::{skill_contact::SkillContact, skill_projection::SkillProjection},
	void_beam::VoidBeam,
	when_traveled_insert::InsertAfterDistanceTraveled,
	Always,
	Movement,
	MovementConfig,
	Once,
	PositionBased,
	VelocityBased,
};
use events::MoveInputEvent;
use std::marker::PhantomData;
use systems::{
	attack::attack,
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
	update_cool_downs::update_cool_downs,
};

pub struct BehaviorsPlugin<TAnimations, TPrefabs, TShaders, TLifeCycles, TInteractions>(
	PhantomData<(TAnimations, TPrefabs, TShaders, TLifeCycles, TInteractions)>,
);

impl<TAnimations, TPrefabs, TShaders, TLifeCycles, TInteractions>
	BehaviorsPlugin<TAnimations, TPrefabs, TShaders, TLifeCycles, TInteractions>
{
	pub fn depends_on(
		_: &TAnimations,
		_: &TPrefabs,
		_: &TShaders,
		_: &TLifeCycles,
		_: &TInteractions,
	) -> Self {
		Self(PhantomData::<(TAnimations, TPrefabs, TShaders, TLifeCycles, TInteractions)>)
	}
}

impl<TAnimationsPlugin, TPrefabsPlugin, TShadersPlugin, TLifeCycles, TInteractionsPlugin> Plugin
	for BehaviorsPlugin<
		TAnimationsPlugin,
		TPrefabsPlugin,
		TShadersPlugin,
		TLifeCycles,
		TInteractionsPlugin,
	>
where
	TAnimationsPlugin: Plugin + HasAnimationsDispatch,
	TPrefabsPlugin: Plugin + RegisterPrefab,
	TShadersPlugin: Plugin + HandlesEffectShading,
	TLifeCycles: Plugin + HandlesDestruction,
	TInteractionsPlugin: Plugin + HandlesInteractions + HandlesEffect<DealDamage>,
{
	fn build(&self, app: &mut App) {
		TPrefabsPlugin::with_dependency::<TInteractionsPlugin>().register_prefab::<VoidBeam>(app);
		TPrefabsPlugin::with_dependency::<(TInteractionsPlugin, TLifeCycles)>()
			.register_prefab::<SkillContact>(app);
		TPrefabsPlugin::register_prefab::<SkillProjection>(app);

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
			.add_systems(Update, GroundTarget::set_position)
			.add_systems(
				Update,
				InsertAfterDistanceTraveled::<TLifeCycles::TDestroy, Velocity>::system,
			)
			.add_systems(Update, SetVelocityForward::system)
			.add_systems(Update, SetPositionAndRotation::<Always>::system)
			.add_systems(Update, SetPositionAndRotation::<Once>::system);
	}
}
