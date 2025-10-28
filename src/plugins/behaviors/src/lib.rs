mod components;
mod input;
mod system_params;
mod systems;
mod traits;

use crate::{
	components::{
		attacking::Attacking,
		fix_points::{Anchor, FixPoints, fix_point::FixPoint},
		skill_usage::SkillUsage,
	},
	system_params::movement::{ReadMovement, WriteMovement},
	systems::face::execute_enemy_face::execute_enemy_face,
};
use bevy::prelude::*;
use common::{
	components::{child_of_persistent::ChildOfPersistent, persistent_entity::PersistentEntity},
	states::game_state::GameState,
	systems::{log::OnError, track_components::TrackComponentInSelfAndChildren},
	traits::{
		animation::{HasAnimationsDispatch, RegisterAnimations},
		handles_agents::HandlesAgents,
		handles_enemies::HandlesEnemies,
		handles_input::{HandlesInput, InputSystemParam},
		handles_movement_behavior::HandlesMovementBehavior,
		handles_orientation::{Face, HandlesOrientation},
		handles_path_finding::HandlesPathFinding,
		handles_physics::{HandlesAllPhysicalEffects, HandlesMotion, HandlesPhysicalObjects},
		handles_player::{
			ConfiguresPlayerMovement,
			HandlesPlayer,
			HandlesPlayerCameras,
			HandlesPlayerMouse,
			PlayerMainCamera,
		},
		handles_saving::HandlesSaving,
		handles_skill_behaviors::{
			Contact,
			HandlesSkillBehaviors,
			Projection,
			SkillEntities,
			SkillRoot,
			SkillSpawner,
		},
		prefab::AddPrefabObserver,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use components::{
	Always,
	Once,
	OverrideFace,
	ground_target::GroundTarget,
	movement::{Movement, path_or_wasd::PathOrWasd},
	set_motion_forward::SetMotionForward,
	skill_behavior::{skill_contact::SkillContact, skill_projection::SkillProjection},
	when_traveled_insert::DestroyAfterDistanceTraveled,
};
use std::marker::PhantomData;
use systems::{
	base_behavior::SelectBehavior,
	chase::ChaseSystem,
	face::{execute_player_face::execute_player_face, get_faces::GetFaces},
	movement::{animate_movement::AnimateMovement, execute_move_update::ExecuteMovement},
	update_count_down::UpdateCountDown,
};

pub struct BehaviorsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TInput, TSaveGame, TAnimations, TPhysics, TPathFinding, TAgents>
	BehaviorsPlugin<(
		TInput,
		TSaveGame,
		TAnimations,
		TPhysics,
		TPathFinding,
		TAgents,
	)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + HasAnimationsDispatch + RegisterAnimations + SystemSetDefinition,
	TPhysics: ThreadSafe + HandlesPhysicalObjects + HandlesAllPhysicalEffects,
	TPathFinding: ThreadSafe + HandlesPathFinding,
	TAgents: ThreadSafe
		+ HandlesPlayer
		+ PlayerMainCamera
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerMovement
		+ HandlesEnemies,
{
	#[allow(clippy::too_many_arguments)]
	pub fn from_plugins(
		_: &TInput,
		_: &TSaveGame,
		_: &TAnimations,
		_: &TPhysics,
		_: &TPathFinding,
		_: &TAgents,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TInput, TSaveGame, TAnimations, TPhysics, TPathFinding, TAgents> Plugin
	for BehaviorsPlugin<(
		TInput,
		TSaveGame,
		TAnimations,
		TPhysics,
		TPathFinding,
		TAgents,
	)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + HasAnimationsDispatch + RegisterAnimations + SystemSetDefinition,
	TPhysics: ThreadSafe + HandlesPhysicalObjects + HandlesMotion + HandlesAllPhysicalEffects,
	TPathFinding: ThreadSafe + HandlesPathFinding,
	TAgents: ThreadSafe
		+ HandlesPlayer
		+ PlayerMainCamera
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerMovement
		+ HandlesEnemies
		+ HandlesAgents,
{
	fn build(&self, app: &mut App) {
		TAnimations::register_movement_direction::<Movement<TPhysics::TMotion>>(app);

		TSaveGame::register_savable_component::<SkillContact>(app);
		TSaveGame::register_savable_component::<SkillProjection>(app);
		TSaveGame::register_savable_component::<Attacking>(app);
		TSaveGame::register_savable_component::<OverrideFace>(app);
		TSaveGame::register_savable_component::<Movement<PathOrWasd<TPhysics::TMotion>>>(app);

		let compute_path = Movement::<PathOrWasd<TPhysics::TMotion>>::compute_path::<
			TPathFinding::TComputePath,
			TPathFinding::TComputerRef,
		>;
		let execute_path = Movement::<PathOrWasd<TPhysics::TMotion>>::execute_movement;
		let execute_movement = Movement::<TPhysics::TMotion>::execute_movement;
		let animate_player_movement = TAgents::TPlayerMovement::animate_movement::<
			Movement<TPhysics::TMotion>,
			TAnimations::TAnimationDispatch,
		>;

		app
			// Required components
			.register_required_components::<TAgents::TPlayer, FixPoints>()
			.register_required_components::<TAgents::TPlayer, SkillUsage>()
			.register_required_components::<TAgents::TEnemy, FixPoints>()
			.register_required_components::<TAgents::TEnemy, SkillUsage>()
			.register_required_components::<SkillContact, TSaveGame::TSaveEntityMarker>()
			.register_required_components::<SkillProjection, TSaveGame::TSaveEntityMarker>()
			// Observers
			.add_prefab_observer::<SkillContact, TPhysics>()
			.add_prefab_observer::<SkillProjection, TPhysics>()
			// Systems
			.add_systems(
				Update,
				(
					// Prep systems
					(
						FixPoint::<SkillSpawner>::insert_in_children_of::<TAgents::TAgent>,
						FixPoints::track_in_self_and_children::<FixPoint<SkillSpawner>>().system(),
					)
						.chain(),
					// Player behaviors
					(
						compute_path,
						execute_path,
						execute_movement,
						animate_player_movement,
						SkillUsage::player::<TAgents::TPlayer, InputSystemParam<TInput>>,
					)
						.chain(),
					// Enemy behaviors
					(
						TAgents::TEnemy::select_behavior::<TAgents::TPlayer>.pipe(OnError::log),
						TAgents::TEnemy::chase::<PathOrWasd<TPhysics::TMotion>>,
						SkillUsage::enemy::<TAgents::TEnemy>,
					)
						.chain(),
					// Skill execution
					(
						Attacking::update::<Virtual>,
						GroundTarget::set_position,
						DestroyAfterDistanceTraveled::system,
						SkillContact::update_range,
						Anchor::<Once>::system.pipe(OnError::log),
						Anchor::<Always>::system.pipe(OnError::log),
						SetMotionForward::system::<TPhysics::TMotion>,
					)
						.chain(),
					// Apply facing
					(
						Movement::<TPhysics::TMotion>::set_faces,
						TAgents::TPlayer::get_faces
							.pipe(execute_player_face::<TAgents::TMouseHover, TAgents::TCamRay>),
						TAgents::TEnemy::get_faces.pipe(execute_enemy_face),
					)
						.chain(),
				)
					.chain()
					.in_set(BehaviorSystems)
					.after(TInput::SYSTEMS)
					.after(TAnimations::SYSTEMS)
					.after(TPathFinding::SYSTEMS)
					.after(TPhysics::SYSTEMS)
					.run_if(in_state(GameState::Play)),
			);
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct BehaviorSystems;

impl<TDependencies> SystemSetDefinition for BehaviorsPlugin<TDependencies> {
	type TSystemSet = BehaviorSystems;

	const SYSTEMS: Self::TSystemSet = BehaviorSystems;
}

impl<TDependencies> HandlesSkillBehaviors for BehaviorsPlugin<TDependencies> {
	type TSkillContact = SkillContact;
	type TSkillProjection = SkillProjection;
	type TSkillUsage = SkillUsage;

	fn spawn_skill(
		commands: &mut ZyheedaCommands,
		contact: Contact,
		projection: Projection,
	) -> SkillEntities {
		let persistent_contact = PersistentEntity::default();
		let contact = commands
			.spawn((SkillContact::from(contact), persistent_contact))
			.id();
		let projection = commands
			.spawn((
				SkillProjection::from(projection),
				ChildOfPersistent(persistent_contact),
			))
			.id();

		SkillEntities {
			root: SkillRoot {
				persistent_entity: persistent_contact,
				entity: contact,
			},
			contact,
			projection,
		}
	}
}

impl<TDependencies> HandlesOrientation for BehaviorsPlugin<TDependencies> {
	type TFaceTemporarily = OverrideFace;

	fn temporarily(face: Face) -> Self::TFaceTemporarily {
		OverrideFace(face)
	}
}

impl<TSettings, TSaveGame, TAnimations, TPhysics, TPathFinding, TAgents> HandlesMovementBehavior
	for BehaviorsPlugin<(
		TSettings,
		TSaveGame,
		TAnimations,
		TPhysics,
		TPathFinding,
		TAgents,
	)>
where
	TPhysics: HandlesMotion,
{
	type TReadMovement<'w, 's> = ReadMovement<'w, 's, TPhysics::TMotion>;
	type TWriteMovement<'w, 's> = WriteMovement<'w, 's, TPhysics::TMotion>;
}
