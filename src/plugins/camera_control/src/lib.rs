mod components;
mod systems;
mod traits;

use crate::{components::camera_arm::CameraArm, systems::move_on_orbit::MoveArmsSystem};
use bevy::prelude::*;
use common::{
	states::game_state::GameState,
	traits::{
		after_plugin::AfterPlugin,
		handles_graphics::HandlesCameras,
		handles_input::HandlesInput,
		handles_player::HandlesPlayer,
		handles_saving::HandlesSaving,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use std::marker::PhantomData;

pub struct CameraControlPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TInput, TPhysics, TSavegame, TPlayers, TGraphics>
	CameraControlPlugin<(TInput, TPhysics, TSavegame, TPlayers, TGraphics)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TPhysics: ThreadSafe + SystemSetDefinition,
	TSavegame: ThreadSafe + HandlesSaving,
	TPlayers: ThreadSafe + HandlesPlayer,
	TGraphics: ThreadSafe + SystemSetDefinition + HandlesCameras,
{
	pub fn from_plugins(
		_: &TInput,
		_: &TPhysics,
		_: &TSavegame,
		_: &TPlayers,
		_: &TGraphics,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TInput, TPhysics, TSavegame, TPlayers, TGraphics> Plugin
	for CameraControlPlugin<(TInput, TPhysics, TSavegame, TPlayers, TGraphics)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TPhysics: ThreadSafe + SystemSetDefinition,
	TSavegame: ThreadSafe + HandlesSaving,
	TPlayers: ThreadSafe + HandlesPlayer,
	TGraphics: ThreadSafe + SystemSetDefinition + HandlesCameras,
{
	fn build(&self, app: &mut App) {
		TSavegame::register_savable_component::<CameraArm>(app);

		app.add_systems(
			Update,
			(
				CameraArm::init_for::<TPlayers::TPlayer>,
				CameraArm::move_arms::<TInput::TInput>,
				CameraArm::apply_direction::<TGraphics::TCameraMut>
					.after_plugin(TPhysics::SYSTEMS)
					.run_if(in_state(GameState::Play)),
			)
				.chain()
				.after_plugin(TInput::SYSTEMS)
				.after_plugin(TGraphics::SYSTEMS)
				.after_plugin(TPhysics::SYSTEMS)
				.run_if(in_state(GameState::Play)),
		);
	}
}
