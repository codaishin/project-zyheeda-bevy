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

pub struct BehaviorsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TInput, TSaveGame, TAnimations, TPhysics, TPathFinding>
	BehaviorsPlugin<(TInput, TSaveGame, TAnimations, TPhysics, TPathFinding)>
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
	for BehaviorsPlugin<(TInput, TSaveGame, TAnimations, TPhysics, TPathFinding)>
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
		TSaveGame::register_savable_component::<Movement<PathOrDirection<TPhysics::TMotion>>>(app);

		let compute_path = MovementDefinition::compute_path::<
			TPhysics::TMotion,
			TPathFinding::TComputePath,
			TPathFinding::TComputerRef,
		>;
		let execute_path =
			MovementDefinition::execute_movement::<Movement<PathOrDirection<TPhysics::TMotion>>>;
		let execute_movement = MovementDefinition::execute_movement::<Movement<TPhysics::TMotion>>;
		let animate_movement_forward = MovementDefinition::animate_movement_forward::<
			Movement<TPhysics::TMotion>,
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
						Movement::<TPhysics::TMotion>::set_faces,
						SetFace::get_faces.pipe(execute_face::<RaycastSystemParam<TPhysics>>),
					)
						.chain(),
					MovementParam::<TPhysics::TMotion>::update_just_removed,
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

impl<TDependencies> HandlesOrientation for BehaviorsPlugin<TDependencies> {
	type TFaceSystemParam<'w, 's> = FaceParamMut<'w, 's>;
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct BehaviorSystems;

impl<TDependencies> SystemSetDefinition for BehaviorsPlugin<TDependencies> {
	type TSystemSet = BehaviorSystems;

	const SYSTEMS: Self::TSystemSet = BehaviorSystems;
}

impl<TInput, TSaveGame, TAnimations, TPhysics, TPathFinding> HandlesMovement
	for BehaviorsPlugin<(TInput, TSaveGame, TAnimations, TPhysics, TPathFinding)>
where
	TPhysics: HandlesMotion,
{
	type TMovement<'w, 's> = MovementParam<'w, 's, TPhysics::TMotion>;
	type TMovementMut<'w, 's> = MovementParamMut<'w, 's, TPhysics::TMotion>;
}
