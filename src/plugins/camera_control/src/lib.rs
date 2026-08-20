mod components;
mod systems;
mod traits;

use crate::{components::camera_arm::CameraArm, systems::move_on_orbit::MoveArmsSystem};
use bevy::prelude::*;
use common::prelude::*;
use std::marker::PhantomData;

pub struct CameraControlPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TGameStates, TInput, TPhysics, TSavegame, TPlayers, TGraphics>
	CameraControlPlugin<(
		TGameStates,
		TInput,
		TPhysics,
		TSavegame,
		TPlayers,
		TGraphics,
	)>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TPhysics: ThreadSafe + SystemSetDefinition,
	TSavegame: ThreadSafe + HandlesSaving,
	TPlayers: ThreadSafe + HandlesPlayer,
	TGraphics: ThreadSafe + SystemSetDefinition + HandlesCameras,
{
	pub fn from_plugins(
		_: &TGameStates,
		_: &TInput,
		_: &TPhysics,
		_: &TSavegame,
		_: &TPlayers,
		_: &TGraphics,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TGameStates, TInput, TPhysics, TSavegame, TPlayers, TGraphics> Plugin
	for CameraControlPlugin<(
		TGameStates,
		TInput,
		TPhysics,
		TSavegame,
		TPlayers,
		TGraphics,
	)>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
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
				CameraArm::apply_direction::<TGraphics::TCameraMut>.after_plugin(TPhysics::SYSTEMS),
			)
				.chain()
				.after_plugin(TGameStates::SYSTEMS)
				.after_plugin(TInput::SYSTEMS)
				.after_plugin(TGraphics::SYSTEMS)
				.after_plugin(TPhysics::SYSTEMS)
				.run_if(not(TGameStates::game_paused())),
		);
	}
}
