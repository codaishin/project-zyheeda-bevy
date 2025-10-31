mod components;
mod system_param;
mod systems;
mod traits;

use crate::{
	components::{
		SetFace,
		attacking::Attacking,
		fix_points::{Anchor, fix_point::FixPointSpawner},
		movement_definition::MovementDefinition,
		skill_usage::SkillUsage,
	},
	system_param::{
		face_param::FaceParamMut,
		movement_param::MovementParamMut,
		skill_param::SkillParamMut,
	},
};
use bevy::prelude::*;
use common::{
	components::{child_of_persistent::ChildOfPersistent, persistent_entity::PersistentEntity},
	states::game_state::GameState,
	systems::log::OnError,
	traits::{
		animation::{HasAnimationsDispatch, RegisterAnimations},
		handles_input::HandlesInput,
		handles_movement::HandlesMovement,
		handles_orientation::HandlesOrientation,
		handles_path_finding::HandlesPathFinding,
		handles_physics::{
			HandlesAllPhysicalEffects,
			HandlesMotion,
			HandlesPhysicalObjects,
			HandlesRaycast,
			RaycastSystemParam,
		},
		handles_saving::HandlesSaving,
		handles_skill_behaviors::{
			Contact,
			HandlesSkillBehaviors,
			Projection,
			SkillEntities,
			SkillRoot,
		},
		handles_skills_control::HandlesSKillControl,
		prefab::AddPrefabObserver,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use components::{
	Always,
	Once,
	SetFaceOverride,
	ground_target::GroundTarget,
	movement::{Movement, path_or_wasd::PathOrWasd},
	set_motion_forward::SetMotionForward,
	skill_behavior::{skill_contact::SkillContact, skill_projection::SkillProjection},
	when_traveled_insert::DestroyAfterDistanceTraveled,
};
use std::marker::PhantomData;
use systems::{face::execute_face::execute_face, update_count_down::UpdateCountDown};

pub struct BehaviorsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TInput, TSaveGame, TAnimations, TPhysics, TPathFinding>
	BehaviorsPlugin<(TInput, TSaveGame, TAnimations, TPhysics, TPathFinding)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + HasAnimationsDispatch + RegisterAnimations + SystemSetDefinition,
	TPhysics: ThreadSafe
		+ HandlesPhysicalObjects
		+ HandlesMotion
		+ HandlesAllPhysicalEffects
		+ HandlesRaycast,
	TPathFinding: ThreadSafe + HandlesPathFinding,
{
	#[allow(clippy::too_many_arguments)]
	pub fn from_plugins(
		_: &TInput,
		_: &TSaveGame,
		_: &TAnimations,
		_: &TPhysics,
		_: &TPathFinding,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TInput, TSaveGame, TAnimations, TPhysics, TPathFinding> Plugin
	for BehaviorsPlugin<(TInput, TSaveGame, TAnimations, TPhysics, TPathFinding)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + HasAnimationsDispatch + RegisterAnimations + SystemSetDefinition,
	TPhysics: ThreadSafe
		+ HandlesPhysicalObjects
		+ HandlesMotion
		+ HandlesAllPhysicalEffects
		+ HandlesRaycast,
	TPathFinding: ThreadSafe + HandlesPathFinding,
{
	fn build(&self, app: &mut App) {
		TAnimations::register_movement_direction::<Movement<TPhysics::TMotion>>(app);

		TSaveGame::register_savable_component::<SkillContact>(app);
		TSaveGame::register_savable_component::<SkillProjection>(app);
		TSaveGame::register_savable_component::<Attacking>(app);
		TSaveGame::register_savable_component::<SetFace>(app);
		TSaveGame::register_savable_component::<SetFaceOverride>(app);
		TSaveGame::register_savable_component::<Movement<PathOrWasd<TPhysics::TMotion>>>(app);

		let compute_path = MovementDefinition::compute_path::<
			TPhysics::TMotion,
			TPathFinding::TComputePath,
			TPathFinding::TComputerRef,
		>;
		let execute_path =
			MovementDefinition::execute_movement::<Movement<PathOrWasd<TPhysics::TMotion>>>;
		let execute_movement = MovementDefinition::execute_movement::<Movement<TPhysics::TMotion>>;
		let animate_movement = MovementDefinition::animate_movement::<
			Movement<TPhysics::TMotion>,
			TAnimations::TAnimationDispatch,
		>;

		app
			// Required components
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
					(FixPointSpawner::insert, SkillUsage::clear_not_refreshed).chain(),
					// Movement
					(
						compute_path,
						execute_path,
						execute_movement,
						animate_movement,
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
						SetFace::get_faces.pipe(execute_face::<RaycastSystemParam<TPhysics>>),
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
	type TFaceSystemParam<'w, 's> = FaceParamMut<'w, 's>;
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct BehaviorSystems;

impl<TDependencies> SystemSetDefinition for BehaviorsPlugin<TDependencies> {
	type TSystemSet = BehaviorSystems;

	const SYSTEMS: Self::TSystemSet = BehaviorSystems;
}

impl<TDependencies> HandlesMovement for BehaviorsPlugin<TDependencies> {
	type TMovementMut<'w, 's> = MovementParamMut<'w, 's>;
}

impl<TDependencies> HandlesSKillControl for BehaviorsPlugin<TDependencies> {
	type TSkillControlMut<'w, 's> = SkillParamMut<'w, 's>;
}
