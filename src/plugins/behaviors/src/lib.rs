pub mod components;
pub mod events;
pub mod traits;

mod systems;

use crate::systems::movement::path::MovementPath;
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	effects::deal_damage::DealDamage,
	labels::Labels,
	states::game_state::GameState,
	systems::log::{log, log_many},
	tools::action_key::movement::MovementKey,
	traits::{
		animation::{HasAnimationsDispatch, RegisterAnimations},
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
			PlayerMainCamera,
		},
		handles_settings::HandlesSettings,
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
	movement::{Movement, path_or_wasd::PathOrWasd, velocity_based::VelocityBased},
	set_position_and_rotation::SetPositionAndRotation,
	set_to_move_forward::SetVelocityForward,
	skill_behavior::{skill_contact::SkillContact, skill_projection::SkillProjection},
	when_traveled_insert::InsertAfterDistanceTraveled,
};
use events::{MoveDirectionalEvent, MovePointerEvent};
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
		trigger_directional_movement_key::TriggerDirectionalMovement,
		trigger_pointer_movement::TriggerPointerMovement,
	},
	update_cool_downs::update_cool_downs,
};

pub struct BehaviorsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSettings, TAnimations, TPrefabs, TLifeCycles, TInteractions, TPathFinding, TEnemies, TPlayers>
	BehaviorsPlugin<(
		TSettings,
		TAnimations,
		TPrefabs,
		TLifeCycles,
		TInteractions,
		TPathFinding,
		TEnemies,
		TPlayers,
	)>
where
	TSettings: ThreadSafe + HandlesSettings,
	TAnimations: ThreadSafe + HasAnimationsDispatch + RegisterAnimations,
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
	#[allow(clippy::too_many_arguments)]
	pub fn depends_on(
		_: &TSettings,
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

impl<TSettings, TAnimations, TPrefabs, TLifeCycles, TInteractions, TPathFinding, TEnemies, TPlayers>
	Plugin
	for BehaviorsPlugin<(
		TSettings,
		TAnimations,
		TPrefabs,
		TLifeCycles,
		TInteractions,
		TPathFinding,
		TEnemies,
		TPlayers,
	)>
where
	TSettings: ThreadSafe + HandlesSettings,
	TAnimations: ThreadSafe + HasAnimationsDispatch + RegisterAnimations,
	TPrefabs: ThreadSafe + RegisterPrefab,
	TLifeCycles: ThreadSafe + HandlesDestruction,
	TInteractions: ThreadSafe + HandlesInteractions + HandlesEffect<DealDamage>,
	TPathFinding: ThreadSafe + HandlesPathFinding,
	TEnemies: ThreadSafe + HandlesEnemies,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ PlayerMainCamera
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerMovement,
{
	fn build(&self, app: &mut App) {
		TPrefabs::with_dependency::<(TInteractions, TLifeCycles)>()
			.register_prefab::<SkillContact>(app);
		TPrefabs::with_dependency::<(TInteractions, TLifeCycles)>()
			.register_prefab::<SkillProjection>(app);
		TAnimations::register_movement_direction::<Movement<VelocityBased>>(app);

		let update_delta = Labels::UPDATE.delta();
		let move_via_pointer = MovePointerEvent::trigger_pointer_movement::<
			TPlayers::TCamRay,
			TSettings::TKeyMap<MovementKey>,
		>;
		let move_via_direction =
			MoveDirectionalEvent::<VelocityBased>::trigger_directional_movement::<
				TPlayers::TPlayerMainCamera,
				TPlayers::TPlayerMovement,
				TSettings::TKeyMap<MovementKey>,
				MovementKey,
			>;

		app.add_event::<MovePointerEvent>()
			.add_event::<MoveDirectionalEvent<VelocityBased>>()
			.add_systems(
				Update,
				(
					move_via_pointer,
					update_delta.pipe(move_via_direction).pipe(log),
					get_faces.pipe(execute_face::<TPlayers::TMouseHover, TPlayers::TCamRay>),
				)
					.chain()
					.run_if(in_state(GameState::Play)),
			)
			.add_systems(Update, update_cool_downs::<Virtual>)
			.add_systems(
				Update,
				(
					Movement::<VelocityBased>::set_faces,
					Movement::<VelocityBased>::cleanup,
					PathOrWasd::<VelocityBased>::cleanup,
				)
					.chain(),
			)
			.add_systems(
				Update,
				(
					TPlayers::TPlayerMovement::set::<
						MovePointerEvent,
						Movement<PathOrWasd<VelocityBased>>,
					>,
					TPlayers::TPlayerMovement::set::<
						MoveDirectionalEvent<VelocityBased>,
						Movement<PathOrWasd<VelocityBased>>,
					>,
					TPlayers::TPlayerMovement::wasd_or_path::<
						VelocityBased,
						TPathFinding::TComputePath,
					>,
					update_delta.pipe(
						TPlayers::TPlayerMovement::execute_movement::<
							Movement<PathOrWasd<VelocityBased>>,
						>,
					),
					update_delta.pipe(
						TPlayers::TPlayerMovement::execute_movement::<Movement<VelocityBased>>,
					),
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
					TEnemies::TEnemy::select_behavior::<TPlayers::TPlayer>.pipe(log_many),
					TEnemies::TEnemy::attack,
					TEnemies::TEnemy::chase::<PathOrWasd<VelocityBased>>,
					TEnemies::TEnemy::wasd_or_path::<VelocityBased, TPathFinding::TComputePath>,
					update_delta.pipe(
						TEnemies::TEnemy::execute_movement::<Movement<PathOrWasd<VelocityBased>>>,
					),
					update_delta
						.pipe(TEnemies::TEnemy::execute_movement::<Movement<VelocityBased>>),
					TEnemies::TEnemy::animate_movement::<
						Movement<VelocityBased>,
						TAnimations::TAnimationDispatch,
					>,
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
