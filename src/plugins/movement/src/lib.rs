mod components;
mod system_param;
mod systems;

#[cfg(debug_assertions)]
mod debug;

use crate::{
	components::{config::SpeedIndex, facing::SetFace, movement::Movement},
	system_param::{
		face_param::FaceParamMut,
		movement_config_param::MovementConfigParamMut,
		movement_param::{MovementParam, MovementParamMut, context_changed::JustRemovedMovements},
	},
	systems::{
		animate_forward::SetForwardAnimationDirection,
		set_movement_facing::SetFaceSystem,
		update_speed::UpdateSpeed,
	},
};
use bevy::prelude::*;
use common::{
	states::game_state::GameState,
	tools::plugin_system_set::PluginSystemSet,
	traits::{
		after_plugin::AfterPlugin,
		handles_animations::HandlesAnimations,
		handles_input::HandlesInput,
		handles_movement::HandlesMovement,
		handles_orientation::HandlesOrientation,
		handles_path_finding::HandlesPathFinding,
		handles_physics::{HandlesAllPhysicalEffects, HandlesMotion, HandlesRaycast},
		handles_saving::HandlesSaving,
		handles_skill_physics::HandlesSkillPhysics,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::facing::SetFaceOverride;
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
		+ SystemSetDefinition
		+ HandlesMotion
		+ HandlesAllPhysicalEffects
		+ HandlesRaycast
		+ HandlesSkillPhysics,
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
		+ SystemSetDefinition
		+ HandlesMotion
		+ HandlesAllPhysicalEffects
		+ HandlesRaycast
		+ HandlesSkillPhysics,
	TPathing: ThreadSafe + HandlesPathFinding,
{
	fn build(&self, app: &mut App) {
		TSaveGame::register_savable_component::<SetFace>(app);
		TSaveGame::register_savable_component::<SetFaceOverride>(app);
		TSaveGame::register_savable_component::<Movement>(app);
		TSaveGame::register_savable_component::<SpeedIndex>(app);

		#[cfg(debug_assertions)]
		debug::draw::<TPhysics::TCharacterMotion>(app);

		app.init_resource::<JustRemovedMovements>().add_systems(
			Update,
			(
				Movement::compute_path::<TPathing::TComputePath, TPathing::TComputerRef>,
				Movement::apply::<TPhysics::TCharacterMotion>,
				TPhysics::TCharacterMotion::update_speed,
				TPhysics::TCharacterMotion::animate_forward::<TAnimations::TAnimationsMut>,
				TPhysics::TCharacterMotion::set_facing,
				SetFace::get_faces.pipe(execute_face::<TPhysics::TRaycastMut, TPhysics::TAgent>),
				MovementParam::<TPhysics::TCharacterMotion>::update_just_removed,
			)
				.chain()
				.in_set(MovementSystems)
				.after_plugin(TInput::SYSTEMS)
				.after_plugin(TAnimations::SYSTEMS)
				.after_plugin(TPathing::SYSTEMS)
				.after_plugin(TPhysics::SYSTEMS)
				.run_if(in_state(GameState::Play)),
		);
	}
}

impl<TDependencies> HandlesOrientation for MovementPlugin<TDependencies> {
	type TFaceSystemParam = FaceParamMut<'static, 'static>;
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct MovementSystems;

impl<TDependencies> SystemSetDefinition for MovementPlugin<TDependencies> {
	type TSystemSet = MovementSystems;

	const SYSTEMS: PluginSystemSet<Self::TSystemSet> = PluginSystemSet::from_set(MovementSystems);
}

impl<TInput, TSaveGame, TAnimations, TPhysics, TPathing> HandlesMovement
	for MovementPlugin<(TInput, TSaveGame, TAnimations, TPhysics, TPathing)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + SystemSetDefinition + HandlesAnimations,
	TPhysics: ThreadSafe
		+ SystemSetDefinition
		+ HandlesMotion
		+ HandlesAllPhysicalEffects
		+ HandlesRaycast,
	TPathing: ThreadSafe + HandlesPathFinding,
{
	type TMovement = MovementParam<'static, 'static, TPhysics::TCharacterMotion>;
	type TMovementMut = MovementParamMut<'static, 'static, TPhysics::TCharacterMotion>;
	type TMovementConfig = MovementConfigParamMut<'static, 'static>;
}
