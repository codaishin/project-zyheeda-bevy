pub mod components;
pub mod input;
pub mod traits;

mod systems;

use crate::{
	components::{
		anchor::{AnchorFixPoints, spawner_fix_point::SpawnerFixPoint},
		on_cool_down::OnCoolDown,
	},
	systems::movement::compute_path::MovementPath,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	components::{child_of_persistent::ChildOfPersistent, persistent_entity::PersistentEntity},
	effects::deal_damage::DealDamage,
	states::game_state::GameState,
	systems::{log::OnError, track_components::TrackComponentInSelfAndChildren},
	tools::action_key::movement::MovementKey,
	traits::{
		animation::{HasAnimationsDispatch, RegisterAnimations},
		delta::Delta,
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
		handles_saving::HandlesSaving,
		handles_settings::HandlesSettings,
		handles_skill_behaviors::{
			Contact,
			HandlesSkillBehaviors,
			Projection,
			SkillEntities,
			SkillRoot,
		},
		prefab::AddPrefabObserver,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::{
	Always,
	Once,
	OverrideFace,
	anchor::Anchor,
	ground_target::GroundTarget,
	movement::{Movement, path_or_wasd::PathOrWasd, velocity_based::VelocityBased},
	set_to_move_forward::SetVelocityForward,
	skill_behavior::{skill_contact::SkillContact, skill_projection::SkillProjection},
	when_traveled_insert::InsertAfterDistanceTraveled,
};
use input::{pointer_input::PointerInput, wasd_input::WasdInput};
use std::marker::PhantomData;
use systems::{
	attack::AttackSystem,
	base_behavior::SelectBehavior,
	chase::ChaseSystem,
	face::{execute_face::execute_face, get_faces::get_faces},
	movement::{
		animate_movement::AnimateMovement,
		execute_move_update::ExecuteMovement,
		insert_process_component::InsertProcessComponent,
		parse_directional_movement_key::ParseDirectionalMovement,
		parse_pointer_movement::ParsePointerMovement,
	},
	update_cool_downs::update_cool_downs,
};

pub struct BehaviorsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<
	TSettings,
	TSaveGame,
	TAnimations,
	TLifeCycles,
	TInteractions,
	TPathFinding,
	TEnemies,
	TPlayers,
>
	BehaviorsPlugin<(
		TSettings,
		TSaveGame,
		TAnimations,
		TLifeCycles,
		TInteractions,
		TPathFinding,
		TEnemies,
		TPlayers,
	)>
where
	TSettings: ThreadSafe + HandlesSettings,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + HasAnimationsDispatch + RegisterAnimations + SystemSetDefinition,
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
	pub fn from_plugins(
		_: &TSettings,
		_: &TSaveGame,
		_: &TAnimations,
		_: &TLifeCycles,
		_: &TInteractions,
		_: &TPathFinding,
		_: &TEnemies,
		_: &TPlayers,
	) -> Self {
		Self(PhantomData)
	}
}

impl<
	TSettings,
	TSaveGame,
	TAnimations,
	TLifeCycles,
	TInteractions,
	TPathFinding,
	TEnemies,
	TPlayers,
> Plugin
	for BehaviorsPlugin<(
		TSettings,
		TSaveGame,
		TAnimations,
		TLifeCycles,
		TInteractions,
		TPathFinding,
		TEnemies,
		TPlayers,
	)>
where
	TSettings: ThreadSafe + HandlesSettings,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + HasAnimationsDispatch + RegisterAnimations + SystemSetDefinition,
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
		TAnimations::register_movement_direction::<Movement<VelocityBased>>(app);
		TSaveGame::register_savable_component::<SkillContact>(app);
		TSaveGame::register_savable_component::<SkillProjection>(app);
		TSaveGame::register_savable_component::<OnCoolDown>(app);
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

		let player_computers_mapping = TPathFinding::computer_mapping_of::<(
			Changed<Movement<PathOrWasd<VelocityBased>>>,
			With<TPlayers::TPlayerMovement>,
		)>();
		let compute_player_path = TPlayers::TPlayerMovement::compute_path::<
			VelocityBased,
			TPathFinding::TComputePath,
			TPathFinding::TComputerRef,
		>();
		let execute_player_path =
			TPlayers::TPlayerMovement::execute_movement::<Movement<PathOrWasd<VelocityBased>>>;
		let execute_player_movement =
			TPlayers::TPlayerMovement::execute_movement::<Movement<VelocityBased>>;
		let animate_player_movement = TPlayers::TPlayerMovement::animate_movement::<
			Movement<VelocityBased>,
			TAnimations::TAnimationDispatch,
		>;

		let enemy_computers_mapping = TPathFinding::computer_mapping_of::<(
			Changed<Movement<PathOrWasd<VelocityBased>>>,
			With<TEnemies::TEnemy>,
		)>();
		let compute_enemy_path = TEnemies::TEnemy::compute_path::<
			VelocityBased,
			TPathFinding::TComputePath,
			TPathFinding::TComputerRef,
		>();
		let execute_enemy_path =
			TEnemies::TEnemy::execute_movement::<Movement<PathOrWasd<VelocityBased>>>;
		let execute_enemy_movement = TEnemies::TEnemy::execute_movement::<Movement<VelocityBased>>;
		let animate_enemy_movement = TEnemies::TEnemy::animate_movement::<
			Movement<VelocityBased>,
			TAnimations::TAnimationDispatch,
		>;

		app
			// Required components
			.register_required_components::<TPlayers::TPlayer, AnchorFixPoints>()
			.register_required_components::<SkillContact, TSaveGame::TSaveEntityMarker>()
			.register_required_components::<SkillProjection, TSaveGame::TSaveEntityMarker>()
			// Observers
			.add_prefab_observer::<SkillContact, (TInteractions, TLifeCycles)>()
			.add_prefab_observer::<SkillProjection, (TInteractions, TLifeCycles)>()
			// Systems
			.add_systems(
				Update,
				(
					// Prep systems
					(
						PathOrWasd::<VelocityBased>::cleanup,
						Movement::<VelocityBased>::cleanup,
						SpawnerFixPoint::insert,
						AnchorFixPoints::track_in_self_and_children::<SpawnerFixPoint>().system(),
					)
						.chain(),
					// Player behaviors
					(
						point_input.pipe(TPlayers::TPlayerMovement::insert_process_component),
						wasd_input.pipe(TPlayers::TPlayerMovement::insert_process_component),
						player_computers_mapping.pipe(compute_player_path.system()),
						Update::delta.pipe(execute_player_path),
						Update::delta.pipe(execute_player_movement),
						animate_player_movement,
					)
						.chain(),
					// Enemy behaviors
					(
						TEnemies::TEnemy::select_behavior::<TPlayers::TPlayer>.pipe(OnError::log),
						TEnemies::TEnemy::attack,
						TEnemies::TEnemy::chase::<PathOrWasd<VelocityBased>>,
						enemy_computers_mapping.pipe(compute_enemy_path.system()),
						Update::delta.pipe(execute_enemy_path),
						Update::delta.pipe(execute_enemy_movement),
						animate_enemy_movement,
					)
						.chain(),
					// Skill execution
					(
						update_cool_downs::<Virtual>,
						GroundTarget::set_position,
						(
							InsertAfterDistanceTraveled::<TLifeCycles::TDestroy, Velocity>::system,
							SkillContact::update_range::<TLifeCycles::TDestroy>,
						)
							.chain(),
						SetVelocityForward::system,
						Anchor::<Always>::system.pipe(OnError::log),
						Anchor::<Once>::system.pipe(OnError::log),
					),
					// Apply facing
					(
						Movement::<VelocityBased>::set_faces,
						get_faces.pipe(execute_face::<TPlayers::TMouseHover, TPlayers::TCamRay>),
					)
						.chain(),
				)
					.chain()
					.in_set(BehaviorSystems)
					.after(TAnimations::SYSTEMS)
					.after(TPathFinding::SYSTEMS)
					.run_if(in_state(GameState::Play)),
			);
	}
}

impl<TDependencies> HandlesSkillBehaviors for BehaviorsPlugin<TDependencies> {
	type TSkillContact = SkillContact;
	type TSkillProjection = SkillProjection;

	fn spawn_skill(
		commands: &mut Commands,
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
