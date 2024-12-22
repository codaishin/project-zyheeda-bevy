pub mod animation;
pub mod components;
pub mod events;
pub mod traits;

mod systems;

use crate::systems::attack::AttackSystem;
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
		handles_enemies::HandlesEnemies,
		handles_interactions::HandlesInteractions,
		handles_orientation::{Face, HandlesOrientation},
		handles_skill_behaviors::{
			HandlesSkillBehaviors,
			Integrity,
			Motion,
			ProjectionOffset,
			Shape,
		},
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::{
	cam_orbit::CamOrbit,
	ground_target::GroundTarget,
	set_position_and_rotation::SetPositionAndRotation,
	set_to_move_forward::SetVelocityForward,
	skill_behavior::{skill_contact::SkillContact, skill_projection::SkillProjection},
	when_traveled_insert::InsertAfterDistanceTraveled,
	Always,
	Movement,
	MovementConfig,
	Once,
	OverrideFace,
	PositionBased,
	VelocityBased,
};
use events::MoveInputEvent;
use std::marker::PhantomData;
use systems::{
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

pub struct BehaviorsPlugin<TAnimations, TPrefabs, TLifeCycles, TInteractions, TEnemies>(
	PhantomData<(TAnimations, TPrefabs, TLifeCycles, TInteractions, TEnemies)>,
);

impl<TAnimations, TPrefabs, TLifeCycles, TInteractions, TEnemies>
	BehaviorsPlugin<TAnimations, TPrefabs, TLifeCycles, TInteractions, TEnemies>
{
	pub fn depends_on(
		_: &TAnimations,
		_: &TPrefabs,
		_: &TLifeCycles,
		_: &TInteractions,
		_: &TEnemies,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TAnimationsPlugin, TPrefabsPlugin, TLifeCycles, TInteractionsPlugin, TEnemies> Plugin
	for BehaviorsPlugin<TAnimationsPlugin, TPrefabsPlugin, TLifeCycles, TInteractionsPlugin, TEnemies>
where
	TAnimationsPlugin: Plugin + HasAnimationsDispatch,
	TPrefabsPlugin: Plugin + RegisterPrefab,
	TLifeCycles: Plugin + HandlesDestruction,
	TInteractionsPlugin: Plugin + HandlesInteractions + HandlesEffect<DealDamage>,
	TEnemies: Plugin + HandlesEnemies,
{
	fn build(&self, app: &mut App) {
		TPrefabsPlugin::with_dependency::<(TInteractionsPlugin, TLifeCycles)>()
			.register_prefab::<SkillContact>(app);
		TPrefabsPlugin::with_dependency::<(TInteractionsPlugin, TLifeCycles)>()
			.register_prefab::<SkillProjection>(app);

		app.add_event::<MoveInputEvent>()
			.add_systems(
				Update,
				(
					trigger_move_input_event::<CamRay>
						.run_if(in_state(MouseContext::<KeyCode>::Default)),
					get_faces.pipe(execute_face::<CamRay>),
				)
					.chain()
					.run_if(in_state(GameState::Play)),
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
			.add_systems(
				Update,
				(chase::<MovementConfig>, TEnemies::TEnemy::attack).chain(),
			)
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

impl<TAnimationsPlugin, TPrefabsPlugin, TLifeCycles, TInteractionsPlugin, TEnemies>
	HandlesSkillBehaviors
	for BehaviorsPlugin<TAnimationsPlugin, TPrefabsPlugin, TLifeCycles, TInteractionsPlugin, TEnemies>
{
	type TSkillContact = SkillContact;
	type TSkillProjection = SkillProjection;

	fn skill_contact(shape: Shape, integrity: Integrity, motion: Motion) -> Self::TSkillContact {
		SkillContact {
			shape,
			integrity,
			motion,
		}
	}

	fn skill_projection(shape: Shape, offset: Option<ProjectionOffset>) -> Self::TSkillProjection {
		SkillProjection { shape, offset }
	}
}

impl<TAnimationsPlugin, TPrefabsPlugin, TLifeCycles, TInteractionsPlugin, TEnemies>
	HandlesOrientation
	for BehaviorsPlugin<TAnimationsPlugin, TPrefabsPlugin, TLifeCycles, TInteractionsPlugin, TEnemies>
{
	type TFaceTemporarily = OverrideFace;

	fn temporarily(face: Face) -> Self::TFaceTemporarily {
		OverrideFace(face)
	}
}
