use bevy::prelude::*;
use common::traits::{
	handles_graphics::InstantiatesCameras,
	handles_player::HandlesPlayer,
	thread_safe::ThreadSafe,
};
use std::marker::PhantomData;

pub struct CameraControlPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TPlayers, TGraphics> Plugin for CameraControlPlugin<(TPlayers, TGraphics)>
where
	TPlayers: ThreadSafe + HandlesPlayer,
	TGraphics: ThreadSafe + InstantiatesCameras,
{
	fn build(&self, _: &mut App) {}
}
