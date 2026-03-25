mod components;
mod observers;
mod system_param;
mod systems;
mod traits;

#[cfg(debug_assertions)]
mod debug;

use crate::{
	components::{
		facing::SetFace,
		ongoing_movement::{IsMoving, OngoingMovement},
	},
	system_param::{
		face_param::FaceParamMut,
		movement_param::{MovementParam, MovementParamMut, context_changed::JustRemovedMovements},
	},
	systems::{
		advance_movement::AdvanceMovement,
		compute_path::ComputePathSystem,
		set_forward_animation_direction::SetForwardAnimationDirection,
	},
};
use bevy::prelude::*;
use common::{
	states::game_state::GameState,
	tools::speed::Speed,
	traits::{
		handles_animations::{AnimationsSystemParamMut, HandlesAnimations},
		handles_input::HandlesInput,
		handles_movement::{HandlesMovement, RequiredClearance},
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
use components::{facing::SetFaceOverride, movement_path::MovementPath};
use std::marker::PhantomData;
use systems::face::execute_face::execute_face;

pub struct MovementPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TInput, TSaveGame, TAnimations, TPhysics, TPathing>
	MovementPlugin<(TInput, TSaveGame, TAnimations, TPhysics, TPathing)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + SystemSetDefinition + HandlesAnimations,
	TPhysics: ThreadSafe
		+ HandlesPhysicalObjects
		+ HandlesMotion
		+ HandlesAllPhysicalEffects
		+ HandlesRaycast,
	TPathing: ThreadSafe + HandlesPathFinding,
{
	#[allow(clippy::too_many_arguments)]
	pub fn from_plugins(
		_: &TInput,
		_: &TSaveGame,
		_: &TAnimations,
		_: &TPhysics,
		_: &TPathing,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TInput, TSaveGame, TAnimations, TPhysics, TPathing> Plugin
	for MovementPlugin<(TInput, TSaveGame, TAnimations, TPhysics, TPathing)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + SystemSetDefinition + HandlesAnimations,
	TPhysics: ThreadSafe
		+ HandlesPhysicalObjects
		+ HandlesMotion
		+ HandlesAllPhysicalEffects
		+ HandlesRaycast,
	TPathing: ThreadSafe + HandlesPathFinding,
{
	fn build(&self, app: &mut App) {
		TSaveGame::register_savable_component::<SetFace>(app);
		TSaveGame::register_savable_component::<SetFaceOverride>(app);
		TSaveGame::register_savable_component::<OngoingMovement>(app);
		TSaveGame::register_savable_component::<MovementPath>(app);

		#[cfg(debug_assertions)]
		debug::draw(app);

		app.init_resource::<JustRemovedMovements>()
			.add_observer(IsMoving::mark)
			.add_observer(IsMoving::unmark_stale)
			.add_systems(
				Update,
				(
					OngoingMovement::set_facing,
					SetFace::get_faces.pipe(execute_face::<RaycastSystemParam<TPhysics>>),
					MovementParam::<TPhysics::TCharacterMotion>::update_just_removed,
				)
					.chain()
					.after(MovementSystems)
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

impl<TInput, TSaveGame, TAnimations, TPhysics, TPathing> HandlesMovement
	for MovementPlugin<(TInput, TSaveGame, TAnimations, TPhysics, TPathing)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + SystemSetDefinition + HandlesAnimations,
	TPhysics: ThreadSafe
		+ HandlesPhysicalObjects
		+ HandlesMotion
		+ HandlesAllPhysicalEffects
		+ HandlesRaycast,
	TPathing: ThreadSafe + HandlesPathFinding,
{
	type TMovement<'w, 's> = MovementParam<'w, 's, TPhysics::TCharacterMotion>;
	type TMovementMut<'w, 's> = MovementParamMut<'w, 's, TPhysics::TCharacterMotion>;

	fn register_movement<TMovementDefinition>(app: &mut App)
	where
		TMovementDefinition: Component,
		for<'a> &'a TMovementDefinition: Into<Speed> + Into<RequiredClearance>,
	{
		app.add_systems(
			Update,
			(
				Changed::<MovementPath>::compute::<
					TPathing::TComputePath,
					TPathing::TComputerRef,
					TMovementDefinition,
				>,
				Without::<IsMoving>::advance::<MovementPath, TMovementDefinition>,
				With::<IsMoving>::advance::<
					(OngoingMovement, TPhysics::TCharacterMotion),
					TMovementDefinition,
				>,
				With::<TMovementDefinition>::set_forward_animation_direction::<
					TPhysics::TCharacterMotion,
					AnimationsSystemParamMut<TAnimations>,
				>,
			)
				.chain()
				.in_set(MovementSystems)
				.after(TInput::SYSTEMS)
				.after(TAnimations::SYSTEMS)
				.after(TPathing::SYSTEMS)
				.after(TPhysics::SYSTEMS)
				.run_if(in_state(GameState::Play)),
		);
	}
}
