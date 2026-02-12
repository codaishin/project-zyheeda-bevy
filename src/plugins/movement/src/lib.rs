mod components;
mod system_param;
mod systems;
mod traits;

use crate::{
	components::{facing::SetFace, movement_definition::MovementDefinition},
	system_param::{
		face_param::FaceParamMut,
		movement_param::{MovementParam, MovementParamMut, context_changed::JustRemovedMovements},
	},
};
use bevy::prelude::*;
use common::{
	states::game_state::GameState,
	traits::{
		handles_animations::{AnimationsSystemParamMut, HandlesAnimations},
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
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::{
	facing::SetFaceOverride,
	movement::{Movement, path_or_direction::PathOrDirection},
};
use std::marker::PhantomData;
use systems::face::execute_face::execute_face;

pub struct MovementPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TInput, TSaveGame, TAnimations, TPhysics, TPathFinding>
	MovementPlugin<(TInput, TSaveGame, TAnimations, TPhysics, TPathFinding)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + SystemSetDefinition + HandlesAnimations,
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
	for MovementPlugin<(TInput, TSaveGame, TAnimations, TPhysics, TPathFinding)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + SystemSetDefinition + HandlesAnimations,
	TPhysics: ThreadSafe
		+ HandlesPhysicalObjects
		+ HandlesMotion
		+ HandlesAllPhysicalEffects
		+ HandlesRaycast,
	TPathFinding: ThreadSafe + HandlesPathFinding,
{
	fn build(&self, app: &mut App) {
		TSaveGame::register_savable_component::<SetFace>(app);
		TSaveGame::register_savable_component::<SetFaceOverride>(app);
		TSaveGame::register_savable_component::<
			Movement<PathOrDirection<TPhysics::TCharacterMotion>, TPhysics::TCharacterImmobilized>,
		>(app);

		let compute_path = MovementDefinition::compute_path::<
			TPhysics::TCharacterMotion,
			TPhysics::TCharacterImmobilized,
			TPathFinding::TComputePath,
			TPathFinding::TComputerRef,
		>;
		let execute_path = MovementDefinition::execute_movement::<
			Movement<PathOrDirection<TPhysics::TCharacterMotion>, TPhysics::TCharacterImmobilized>,
		>;
		let execute_movement = MovementDefinition::execute_movement::<
			Movement<TPhysics::TCharacterMotion, TPhysics::TCharacterImmobilized>,
		>;
		let animate_movement_forward = MovementDefinition::animate_movement_forward::<
			Movement<TPhysics::TCharacterMotion, TPhysics::TCharacterImmobilized>,
			AnimationsSystemParamMut<TAnimations>,
		>;

		app
			// Resources
			.init_resource::<JustRemovedMovements>()
			// Systems
			.add_systems(
				Update,
				(
					// Movement
					(
						compute_path,
						execute_path,
						execute_movement,
						animate_movement_forward,
					)
						.chain(),
					// Apply facing
					(
						Movement::<TPhysics::TCharacterMotion, TPhysics::TCharacterImmobilized>::set_faces,
						SetFace::get_faces.pipe(execute_face::<RaycastSystemParam<TPhysics>>),
					)
						.chain(),
					MovementParam::<TPhysics::TCharacterMotion, TPhysics::TCharacterImmobilized>::update_just_removed,
				)
					.chain()
					.in_set(MovementSystems)
					.after(TInput::SYSTEMS)
					.after(TAnimations::SYSTEMS)
					.after(TPathFinding::SYSTEMS)
					.after(TPhysics::SYSTEMS)
					.run_if(in_state(GameState::Play)),
			);
	}
}

impl<TDependencies> HandlesOrientation for MovementPlugin<TDependencies> {
	type TFaceSystemParam<'w, 's> = FaceParamMut<'w, 's>;
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct MovementSystems;

impl<TDependencies> SystemSetDefinition for MovementPlugin<TDependencies> {
	type TSystemSet = MovementSystems;

	const SYSTEMS: Self::TSystemSet = MovementSystems;
}

impl<TInput, TSaveGame, TAnimations, TPhysics, TPathFinding> HandlesMovement
	for MovementPlugin<(TInput, TSaveGame, TAnimations, TPhysics, TPathFinding)>
where
	TPhysics: HandlesMotion,
{
	type TMovement<'w, 's> =
		MovementParam<'w, 's, TPhysics::TCharacterMotion, TPhysics::TCharacterImmobilized>;
	type TMovementMut<'w, 's> =
		MovementParamMut<'w, 's, TPhysics::TCharacterMotion, TPhysics::TCharacterImmobilized>;
}
