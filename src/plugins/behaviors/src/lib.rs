mod components;
mod input;
mod systems;
mod traits;

use crate::{
	components::{
		attacking::Attacking,
		fix_points::{Anchor, FixPoints, fix_point::FixPoint},
		skill_usage::SkillUsage,
	},
	systems::{
		face::execute_enemy_face::execute_enemy_face,
		movement::{compute_path::MovementPath, insert_process_component::ProcessInput},
	},
};
use bevy::prelude::*;
use common::{
	components::{child_of_persistent::ChildOfPersistent, persistent_entity::PersistentEntity},
	states::game_state::GameState,
	systems::{log::OnError, track_components::TrackComponentInSelfAndChildren},
	tools::action_key::{movement::MovementKey, slot::PlayerSlot},
	traits::{
		animation::{HasAnimationsDispatch, RegisterAnimations},
		handles_enemies::HandlesEnemies,
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
		handles_settings::HandlesSettings,
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
use input::{pointer_input::PointerInput, wasd_input::WasdInput};
use std::marker::PhantomData;
use systems::{
	base_behavior::SelectBehavior,
	chase::ChaseSystem,
	face::{execute_player_face::execute_player_face, get_faces::GetFaces},
	movement::{
		animate_movement::AnimateMovement,
		execute_move_update::ExecuteMovement,
		insert_process_component::InsertProcessComponent,
		parse_directional_movement_key::ParseDirectionalMovement,
		parse_pointer_movement::ParsePointerMovement,
	},
	update_count_down::UpdateCountDown,
};

pub struct BehaviorsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSettings, TSaveGame, TAnimations, TPhysics, TPathFinding, TAgents>
	BehaviorsPlugin<(
		TSettings,
		TSaveGame,
		TAnimations,
		TPhysics,
		TPathFinding,
		TAgents,
	)>
where
	TSettings: ThreadSafe + HandlesSettings,
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
		_: &TSettings,
		_: &TSaveGame,
		_: &TAnimations,
		_: &TPhysics,
		_: &TPathFinding,
		_: &TAgents,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TSettings, TSaveGame, TAnimations, TPhysics, TPathFinding, TAgents> Plugin
	for BehaviorsPlugin<(
		TSettings,
		TSaveGame,
		TAnimations,
		TPhysics,
		TPathFinding,
		TAgents,
	)>
where
	TSettings: ThreadSafe + HandlesSettings,
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
		+ HandlesEnemies,
{
	fn build(&self, app: &mut App) {
		TAnimations::register_movement_direction::<Movement<TPhysics::TMotion>>(app);

		TSaveGame::register_savable_component::<SkillContact>(app);
		TSaveGame::register_savable_component::<SkillProjection>(app);
		TSaveGame::register_savable_component::<Attacking>(app);
		TSaveGame::register_savable_component::<OverrideFace>(app);
		TSaveGame::register_savable_component::<Movement<PathOrWasd<TPhysics::TMotion>>>(app);

		let point_input = PointerInput::<TPhysics::TMotion>::parse::<
			TAgents::TCamRay,
			TSettings::TKeyMap<MovementKey>,
		>;
		let wasd_input = WasdInput::<TPhysics::TMotion>::parse::<
			TAgents::TPlayerMainCamera,
			TSettings::TKeyMap<MovementKey>,
			TAgents::TPlayer,
		>;
		let wasd_input = wasd_input.pipe(OnError::log_and_return(|| ProcessInput::None));

		let compute_player_path = TAgents::TPlayerMovement::compute_path::<
			TPhysics::TMotion,
			TPathFinding::TComputePath,
			TPathFinding::TComputerRef,
		>;
		let execute_player_path =
			TAgents::TPlayerMovement::execute_movement::<Movement<PathOrWasd<TPhysics::TMotion>>>;
		let execute_player_movement =
			TAgents::TPlayerMovement::execute_movement::<Movement<TPhysics::TMotion>>;
		let animate_player_movement = TAgents::TPlayerMovement::animate_movement::<
			Movement<TPhysics::TMotion>,
			TAnimations::TAnimationDispatch,
		>;

		let compute_enemy_path = TAgents::TEnemy::compute_path::<
			TPhysics::TMotion,
			TPathFinding::TComputePath,
			TPathFinding::TComputerRef,
		>;
		let execute_enemy_path =
			TAgents::TEnemy::execute_movement::<Movement<PathOrWasd<TPhysics::TMotion>>>;
		let execute_enemy_movement =
			TAgents::TEnemy::execute_movement::<Movement<TPhysics::TMotion>>;
		let animate_enemy_movement = TAgents::TEnemy::animate_movement::<
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
						FixPoint::<SkillSpawner>::insert_in_children_of::<TAgents::TPlayer>,
						FixPoint::<SkillSpawner>::insert_in_children_of::<TAgents::TEnemy>,
						FixPoints::track_in_self_and_children::<FixPoint<SkillSpawner>>().system(),
					)
						.chain(),
					// Player behaviors
					(
						point_input.pipe(TAgents::TPlayer::insert_process_component),
						wasd_input.pipe(TAgents::TPlayer::insert_process_component),
						compute_player_path,
						execute_player_path,
						execute_player_movement,
						animate_player_movement,
						SkillUsage::player::<TAgents::TPlayer, TSettings::TKeyMap<PlayerSlot>>,
					)
						.chain(),
					// Enemy behaviors
					(
						TAgents::TEnemy::select_behavior::<TAgents::TPlayer>.pipe(OnError::log),
						TAgents::TEnemy::chase::<PathOrWasd<TPhysics::TMotion>>,
						compute_enemy_path,
						execute_enemy_path,
						execute_enemy_movement,
						animate_enemy_movement,
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
					.after(TAnimations::SYSTEMS)
					.after(TPathFinding::SYSTEMS)
					.after(TPhysics::SYSTEMS)
					.run_if(in_state(GameState::Play)),
			);
	}
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

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct BehaviorSystems;

impl<TDependencies> SystemSetDefinition for BehaviorsPlugin<TDependencies> {
	type TSystemSet = BehaviorSystems;

	const SYSTEMS: Self::TSystemSet = BehaviorSystems;
}
