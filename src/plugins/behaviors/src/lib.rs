pub mod components;
pub mod events;
pub mod traits;

mod systems;

use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	effects::deal_damage::DealDamage,
	labels::Labels,
	states::{game_state::GameState, mouse_context::MouseContext},
	systems::insert_required::{InsertOn, InsertRequired},
	traits::{
		animation::HasAnimationsDispatch,
		handles_destruction::HandlesDestruction,
		handles_effect::HandlesEffect,
		handles_enemies::HandlesEnemies,
		handles_interactions::HandlesInteractions,
		handles_orientation::{Face, HandlesOrientation},
		handles_path_finding::HandlesPathFinding,
		handles_player::{
			ConfiguresPlayerMovement,
			HandlesPlayer,
			HandlesPlayerCameras,
			HandlesPlayerMouse,
		},
		handles_skill_behaviors::{
			HandlesSkillBehaviors,
			Integrity,
			Motion,
			ProjectionOffset,
			Shape,
		},
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
		thread_safe::ThreadSafe,
	},
};
use components::{
	Always,
	Once,
	OverrideFace,
	ground_target::GroundTarget,
	movement::{Movement, along_path::AlongPath, velocity_based::VelocityBased},
	set_position_and_rotation::SetPositionAndRotation,
	set_to_move_forward::SetVelocityForward,
	skill_behavior::{skill_contact::SkillContact, skill_projection::SkillProjection},
	when_traveled_insert::InsertAfterDistanceTraveled,
};
use events::MoveInputEvent;
use std::marker::PhantomData;
use systems::{
	attack::AttackSystem,
	base_behavior::SelectBehavior,
	chase::ChaseSystem,
	face::{execute_face::execute_face, get_faces::get_faces},
	movement::{
		animate_movement::AnimateMovement,
		execute_move_update::ExecuteMovement,
		set_player_movement::SetPlayerMovement,
		trigger_event::trigger_move_input_event,
	},
	update_cool_downs::update_cool_downs,
};

pub struct BehaviorsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TAnimations, TPrefabs, TLifeCycles, TInteractions, TPathFinding, TEnemies, TPlayers>
	BehaviorsPlugin<(
		TAnimations,
		TPrefabs,
		TLifeCycles,
		TInteractions,
		TPathFinding,
		TEnemies,
		TPlayers,
	)>
where
	TAnimations: ThreadSafe + HasAnimationsDispatch,
	TPrefabs: ThreadSafe + RegisterPrefab,
	TLifeCycles: ThreadSafe + HandlesDestruction,
	TInteractions: ThreadSafe + HandlesInteractions + HandlesEffect<DealDamage>,
	TPathFinding: ThreadSafe + HandlesPathFinding,
	TEnemies: ThreadSafe + HandlesEnemies,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerMovement,
{
	pub fn depends_on(
		_: &TAnimations,
		_: &TPrefabs,
		_: &TLifeCycles,
		_: &TInteractions,
		_: &TPathFinding,
		_: &TEnemies,
		_: &TPlayers,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TAnimations, TPrefabs, TLifeCycles, TInteractions, TPathFinding, TEnemies, TPlayers> Plugin
	for BehaviorsPlugin<(
		TAnimations,
		TPrefabs,
		TLifeCycles,
		TInteractions,
		TPathFinding,
		TEnemies,
		TPlayers,
	)>
where
	TAnimations: ThreadSafe + HasAnimationsDispatch,
	TPrefabs: ThreadSafe + RegisterPrefab,
	TLifeCycles: ThreadSafe + HandlesDestruction,
	TInteractions: ThreadSafe + HandlesInteractions + HandlesEffect<DealDamage>,
	TPathFinding: ThreadSafe + HandlesPathFinding,
	TEnemies: ThreadSafe + HandlesEnemies,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerMovement,
{
	fn build(&self, app: &mut App) {
		TPrefabs::with_dependency::<(TInteractions, TLifeCycles)>()
			.register_prefab::<SkillContact>(app);
		TPrefabs::with_dependency::<(TInteractions, TLifeCycles)>()
			.register_prefab::<SkillProjection>(app);

		app.add_event::<MoveInputEvent>()
			.add_systems(
				Update,
				(
					trigger_move_input_event::<TPlayers::TCamRay>
						.run_if(in_state(MouseContext::<KeyCode>::Default)),
					get_faces.pipe(execute_face::<TPlayers::TMouseHover, TPlayers::TCamRay>),
				)
					.chain()
					.run_if(in_state(GameState::Play)),
			)
			.add_systems(Update, update_cool_downs::<Virtual>)
			.add_systems(
				Labels::PREFAB_INSTANTIATION.label(),
				(
					InsertOn::<Movement<AlongPath<VelocityBased>>>::required(
						|Movement { target, .. }| AlongPath::<VelocityBased>::to(*target),
					),
					AlongPath::<VelocityBased>::set_path::<TPathFinding::TComputePath>,
				)
					.chain(),
			)
			.add_systems(
				Update,
				(
					Movement::<VelocityBased>::set_faces,
					Movement::<VelocityBased>::cleanup,
				)
					.chain(),
			)
			.add_systems(
				Update,
				(
					TPlayers::TPlayerMovement::set::<Movement<AlongPath<VelocityBased>>>,
					TPlayers::TPlayerMovement::execute_movement::<Movement<AlongPath<VelocityBased>>>,
					TPlayers::TPlayerMovement::execute_movement::<Movement<VelocityBased>>,
					TPlayers::TPlayerMovement::animate_movement::<
						Movement<VelocityBased>,
						TAnimations::TAnimationDispatch,
					>,
				)
					.chain(),
			)
			.add_systems(
				Update,
				(
					TEnemies::TEnemy::select_behavior::<TPlayers::TPlayer>,
					TEnemies::TEnemy::chase,
					TEnemies::TEnemy::animate_movement::<
						Movement<VelocityBased>,
						TAnimations::TAnimationDispatch,
					>,
					TEnemies::TEnemy::execute_movement::<Movement<VelocityBased>>,
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

impl<TDependencies> HandlesSkillBehaviors for BehaviorsPlugin<TDependencies> {
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

impl<TDependencies> HandlesOrientation for BehaviorsPlugin<TDependencies> {
	type TFaceTemporarily = OverrideFace;

	fn temporarily(face: Face) -> Self::TFaceTemporarily {
		OverrideFace(face)
	}
}
