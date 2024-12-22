pub mod components;
pub mod events;
pub mod traits;

mod systems;

use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	components::MainCamera,
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
		handles_player::{ConfiguresPlayerMovement, HandlesPlayer},
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
	Once,
	OverrideFace,
	VelocityBased,
};
use events::MoveInputEvent;
use std::marker::PhantomData;
use systems::{
	attack::AttackSystem,
	base_behavior::SelectBehavior,
	chase::ChaseSystem,
	cleanup::cleanup,
	face::{execute_face::execute_face, get_faces::get_faces},
	movement::{
		animate_movement::AnimateMovement,
		execute_move_velocity_based::ExecuteMovement,
		move_on_orbit::move_on_orbit,
		move_with_target::move_with_target,
		set_camera_to_orbit_player::SetCameraToOrbit,
		set_player_movement::SetPlayerMovement,
		trigger_event::trigger_move_input_event,
	},
	update_cool_downs::update_cool_downs,
};

pub struct BehaviorsPlugin<TAnimations, TPrefabs, TLifeCycles, TInteractions, TEnemies, TPlayer>(
	PhantomData<(
		TAnimations,
		TPrefabs,
		TLifeCycles,
		TInteractions,
		TEnemies,
		TPlayer,
	)>,
);

impl<TAnimations, TPrefabs, TLifeCycles, TInteractions, TEnemies, TPlayer>
	BehaviorsPlugin<TAnimations, TPrefabs, TLifeCycles, TInteractions, TEnemies, TPlayer>
{
	pub fn depends_on(
		_: &TAnimations,
		_: &TPrefabs,
		_: &TLifeCycles,
		_: &TInteractions,
		_: &TEnemies,
		_: &TPlayer,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TAnimationsPlugin, TPrefabsPlugin, TLifeCycles, TInteractionsPlugin, TEnemies, TPlayers> Plugin
	for BehaviorsPlugin<
		TAnimationsPlugin,
		TPrefabsPlugin,
		TLifeCycles,
		TInteractionsPlugin,
		TEnemies,
		TPlayers,
	>
where
	TAnimationsPlugin: Plugin + HasAnimationsDispatch,
	TPrefabsPlugin: Plugin + RegisterPrefab,
	TLifeCycles: Plugin + HandlesDestruction,
	TInteractionsPlugin: Plugin + HandlesInteractions + HandlesEffect<DealDamage>,
	TEnemies: Plugin + HandlesEnemies,
	TPlayers: Plugin + HandlesPlayer + ConfiguresPlayerMovement,
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
			.add_systems(Update, MainCamera::set_camera_to_orbit::<TPlayers::TPlayer>)
			.add_systems(
				Update,
				(
					TPlayers::TPlayerMovement::set_movement,
					TPlayers::TPlayerMovement::animate_movement::<
						Movement<VelocityBased>,
						TAnimationsPlugin::TAnimationDispatch,
					>,
					TPlayers::TPlayerMovement::execute_movement::<Movement<VelocityBased>>
						.pipe(cleanup),
				),
			)
			.add_systems(
				Update,
				(
					TEnemies::TEnemy::select_behavior::<TPlayers::TPlayer>,
					TEnemies::TEnemy::chase,
					TEnemies::TEnemy::animate_movement::<
						Movement<VelocityBased>,
						TAnimationsPlugin::TAnimationDispatch,
					>,
					TEnemies::TEnemy::execute_movement::<Movement<VelocityBased>>.pipe(cleanup),
					TEnemies::TEnemy::attack,
				)
					.chain(),
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

impl<TAnimationsPlugin, TPrefabsPlugin, TLifeCycles, TInteractionsPlugin, TEnemies, TPlayers>
	HandlesSkillBehaviors
	for BehaviorsPlugin<
		TAnimationsPlugin,
		TPrefabsPlugin,
		TLifeCycles,
		TInteractionsPlugin,
		TEnemies,
		TPlayers,
	>
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

impl<TAnimationsPlugin, TPrefabsPlugin, TLifeCycles, TInteractionsPlugin, TEnemies, TPlayers>
	HandlesOrientation
	for BehaviorsPlugin<
		TAnimationsPlugin,
		TPrefabsPlugin,
		TLifeCycles,
		TInteractionsPlugin,
		TEnemies,
		TPlayers,
	>
{
	type TFaceTemporarily = OverrideFace;

	fn temporarily(face: Face) -> Self::TFaceTemporarily {
		OverrideFace(face)
	}
}
