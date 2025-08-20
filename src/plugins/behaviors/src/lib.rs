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
	systems::{face::execute_enemy_face::execute_enemy_face, movement::compute_path::MovementPath},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	components::{child_of_persistent::ChildOfPersistent, persistent_entity::PersistentEntity},
	effects::deal_damage::DealDamage,
	states::game_state::GameState,
	systems::{log::OnError, track_components::TrackComponentInSelfAndChildren},
	tools::action_key::{movement::MovementKey, slot::PlayerSlot},
	traits::{
		animation::{HasAnimationsDispatch, RegisterAnimations},
		delta::Delta,
		handles_effect::HandlesEffect,
		handles_enemies::HandlesEnemyConfig,
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
	movement::{Movement, path_or_wasd::PathOrWasd, velocity_based::VelocityBased},
	set_to_move_forward::SetVelocityForward,
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

impl<TSettings, TSaveGame, TAnimations, TInteractions, TPathFinding, TEnemies, TPlayers>
	BehaviorsPlugin<(
		TSettings,
		TSaveGame,
		TAnimations,
		TInteractions,
		TPathFinding,
		TEnemies,
		TPlayers,
	)>
where
	TSettings: ThreadSafe + HandlesSettings,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + HasAnimationsDispatch + RegisterAnimations + SystemSetDefinition,
	TInteractions: ThreadSafe + HandlesInteractions + HandlesEffect<DealDamage>,
	TPathFinding: ThreadSafe + HandlesPathFinding,
	TEnemies: ThreadSafe + HandlesEnemyConfig,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ PlayerMainCamera
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerMovement,
{
	#[allow(clippy::too_many_arguments)]
	pub fn from_plugins(
		_: &TSettings,
		_: &TSaveGame,
		_: &TAnimations,
		_: &TInteractions,
		_: &TPathFinding,
		_: &TEnemies,
		_: &TPlayers,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TSettings, TSaveGame, TAnimations, TInteractions, TPathFinding, TEnemies, TPlayers> Plugin
	for BehaviorsPlugin<(
		TSettings,
		TSaveGame,
		TAnimations,
		TInteractions,
		TPathFinding,
		TEnemies,
		TPlayers,
	)>
where
	TSettings: ThreadSafe + HandlesSettings,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + HasAnimationsDispatch + RegisterAnimations + SystemSetDefinition,
	TInteractions: ThreadSafe + HandlesInteractions + HandlesEffect<DealDamage>,
	TPathFinding: ThreadSafe + HandlesPathFinding,
	TEnemies: ThreadSafe + HandlesEnemyConfig,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ PlayerMainCamera
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerMovement,
{
	fn build(&self, app: &mut App) {
		TAnimations::register_movement_direction::<Movement<VelocityBased>>(app);
		TSaveGame::register_savable_component::<SkillContact>(app);
		TSaveGame::register_savable_component::<SkillProjection>(app);
		TSaveGame::register_savable_component::<Attacking>(app);
		TSaveGame::register_savable_component::<Movement<PathOrWasd<VelocityBased>>>(app);
		TSaveGame::register_savable_component::<OverrideFace>(app);

		let point_input = PointerInput::parse::<TPlayers::TCamRay, TSettings::TKeyMap<MovementKey>>;
		let wasd_input = WasdInput::<VelocityBased>::parse::<
			TPlayers::TPlayerMainCamera,
			TPlayers::TPlayerMovement,
			TSettings::TKeyMap<MovementKey>,
			MovementKey,
		>;
		let wasd_input = Update::delta
			.pipe(wasd_input)
			.pipe(OnError::log_and_return(|| None));

		let compute_player_path = TPlayers::TPlayerMovement::compute_path::<
			VelocityBased,
			TPathFinding::TComputePath,
			TPathFinding::TComputerRef,
		>;
		let execute_player_path =
			TPlayers::TPlayerMovement::execute_movement::<Movement<PathOrWasd<VelocityBased>>>;
		let execute_player_movement =
			TPlayers::TPlayerMovement::execute_movement::<Movement<VelocityBased>>;
		let animate_player_movement = TPlayers::TPlayerMovement::animate_movement::<
			Movement<VelocityBased>,
			TAnimations::TAnimationDispatch,
		>;

		let compute_enemy_path = TEnemies::TEnemyBehavior::compute_path::<
			VelocityBased,
			TPathFinding::TComputePath,
			TPathFinding::TComputerRef,
		>;
		let execute_enemy_path =
			TEnemies::TEnemyBehavior::execute_movement::<Movement<PathOrWasd<VelocityBased>>>;
		let execute_enemy_movement =
			TEnemies::TEnemyBehavior::execute_movement::<Movement<VelocityBased>>;
		let animate_enemy_movement = TEnemies::TEnemyBehavior::animate_movement::<
			Movement<VelocityBased>,
			TAnimations::TAnimationDispatch,
		>;

		app
			// Required components
			.register_required_components::<TPlayers::TPlayer, FixPoints>()
			.register_required_components::<TPlayers::TPlayer, SkillUsage>()
			.register_required_components::<TEnemies::TEnemyBehavior, FixPoints>()
			.register_required_components::<TEnemies::TEnemyBehavior, SkillUsage>()
			.register_required_components::<SkillContact, TSaveGame::TSaveEntityMarker>()
			.register_required_components::<SkillProjection, TSaveGame::TSaveEntityMarker>()
			// Observers
			.add_prefab_observer::<SkillContact, TInteractions>()
			.add_prefab_observer::<SkillProjection, TInteractions>()
			// Systems
			.add_systems(
				Update,
				(
					// Prep systems
					(
						PathOrWasd::<VelocityBased>::cleanup,
						Movement::<VelocityBased>::cleanup,
						FixPoint::<SkillSpawner>::insert_in_children_of::<TPlayers::TPlayer>,
						FixPoint::<SkillSpawner>::insert_in_children_of::<TEnemies::TEnemyBehavior>,
						FixPoints::track_in_self_and_children::<FixPoint<SkillSpawner>>().system(),
					)
						.chain(),
					// Player behaviors
					(
						point_input.pipe(TPlayers::TPlayerMovement::insert_process_component),
						wasd_input.pipe(TPlayers::TPlayerMovement::insert_process_component),
						compute_player_path,
						Update::delta.pipe(execute_player_path),
						Update::delta.pipe(execute_player_movement),
						animate_player_movement,
						SkillUsage::player::<TPlayers::TPlayer, TSettings::TKeyMap<PlayerSlot>>,
					)
						.chain(),
					// Enemy behaviors
					(
						TEnemies::TEnemyBehavior::select_behavior::<TPlayers::TPlayer>
							.pipe(OnError::log),
						TEnemies::TEnemyBehavior::chase::<PathOrWasd<VelocityBased>>,
						compute_enemy_path,
						Update::delta.pipe(execute_enemy_path),
						Update::delta.pipe(execute_enemy_movement),
						animate_enemy_movement,
						SkillUsage::enemy::<TEnemies::TEnemyBehavior>,
					)
						.chain(),
					// Skill execution
					(
						Attacking::update::<Virtual>,
						GroundTarget::set_position,
						DestroyAfterDistanceTraveled::<Velocity>::system,
						SkillContact::update_range,
						Anchor::<Once>::system.pipe(OnError::log),
						Anchor::<Always>::system.pipe(OnError::log),
						SetVelocityForward::system,
					)
						.chain(),
					// Apply facing
					(
						Movement::<VelocityBased>::set_faces,
						TPlayers::TPlayer::get_faces
							.pipe(execute_player_face::<TPlayers::TMouseHover, TPlayers::TCamRay>),
						TEnemies::TEnemyBehavior::get_faces.pipe(execute_enemy_face),
					)
						.chain(),
				)
					.chain()
					.in_set(BehaviorSystems)
					.after(TAnimations::SYSTEMS)
					.after(TPathFinding::SYSTEMS)
					.after(TInteractions::SYSTEMS)
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
