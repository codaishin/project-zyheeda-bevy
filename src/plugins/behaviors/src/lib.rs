pub mod components;
pub mod input;
pub mod traits;

mod systems;

use crate::{
	components::anchor::{AnchorFixPoints, spawner_fix_point::SpawnerFixPoint},
	systems::movement::compute_path::MovementPath,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	effects::deal_damage::DealDamage,
	states::game_state::GameState,
	systems::{
		log::{log_many, log_or_unwrap_option},
		track_components::TrackComponentInSelfAndChildren,
	},
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
		handles_settings::HandlesSettings,
		handles_skill_behaviors::{Contact, HandlesSkillBehaviors, Projection, SkillEntities},
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

impl<TSettings, TAnimations, TLifeCycles, TInteractions, TPathFinding, TEnemies, TPlayers>
	BehaviorsPlugin<(
		TSettings,
		TAnimations,
		TLifeCycles,
		TInteractions,
		TPathFinding,
		TEnemies,
		TPlayers,
	)>
where
	TSettings: ThreadSafe + HandlesSettings,
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

impl<TSettings, TAnimations, TLifeCycles, TInteractions, TPathFinding, TEnemies, TPlayers> Plugin
	for BehaviorsPlugin<(
		TSettings,
		TAnimations,
		TLifeCycles,
		TInteractions,
		TPathFinding,
		TEnemies,
		TPlayers,
	)>
where
	TSettings: ThreadSafe + HandlesSettings,
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

		let point_input = PointerInput::parse::<TPlayers::TCamRay, TSettings::TKeyMap<MovementKey>>;
		let wasd_input = WasdInput::<VelocityBased>::parse::<
			TPlayers::TPlayerMainCamera,
			TPlayers::TPlayerMovement,
			TSettings::TKeyMap<MovementKey>,
			MovementKey,
		>;
		let wasd_input = Update::delta.pipe(wasd_input).pipe(log_or_unwrap_option);

		let compute_player_path =
			TPlayers::TPlayerMovement::compute_path::<VelocityBased, TPathFinding::TComputePath>;
		let execute_player_path =
			TPlayers::TPlayerMovement::execute_movement::<Movement<PathOrWasd<VelocityBased>>>;
		let execute_player_movement =
			TPlayers::TPlayerMovement::execute_movement::<Movement<VelocityBased>>;
		let animate_player_movement = TPlayers::TPlayerMovement::animate_movement::<
			Movement<VelocityBased>,
			TAnimations::TAnimationDispatch,
		>;

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
						compute_player_path,
						Update::delta.pipe(execute_player_path),
						Update::delta.pipe(execute_player_movement),
						animate_player_movement,
					)
						.chain(),
					// Enemy behaviors
					(
						TEnemies::TEnemy::select_behavior::<TPlayers::TPlayer>.pipe(log_many),
						TEnemies::TEnemy::attack,
						TEnemies::TEnemy::chase::<PathOrWasd<VelocityBased>>,
						TEnemies::TEnemy::compute_path::<VelocityBased, TPathFinding::TComputePath>,
						Update::delta.pipe(execute_enemy_path),
						Update::delta.pipe(execute_enemy_movement),
						animate_enemy_movement,
					)
						.chain(),
					// Skill execution
					(
						update_cool_downs::<Virtual>,
						GroundTarget::set_position,
						InsertAfterDistanceTraveled::<TLifeCycles::TDestroy, Velocity>::system,
						SetVelocityForward::system,
						Anchor::<Always>::system.pipe(log_many),
						Anchor::<Once>::system.pipe(log_many),
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
					/* FIXME: `.before()` should synch facing and animation weights, but for some reason it
					 *         doesn't. Using `.after()` might cause a one frame delay here, but seems to
					 *         ensure that animation weights are computed off of correct look direction after
					 *         applying transform facing.
					 */
					.after(TAnimations::SYSTEMS)
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
		let contact = commands.spawn(SkillContact::from(contact)).id();
		let projection = commands
			.spawn((SkillProjection::from(projection), ChildOf(contact)))
			.id();

		SkillEntities {
			root: contact,
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
