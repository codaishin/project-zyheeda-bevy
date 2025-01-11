use bevy::prelude::*;
use common::traits::{
	handles_graphics::PlayerCameras,
	handles_player::{HandlesPlayer, PlayerMainCamera},
	thread_safe::ThreadSafe,
};
use std::marker::PhantomData;

pub struct CameraControlPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TPlayers, TGraphics> Plugin for CameraControlPlugin<(TPlayers, TGraphics)>
where
	TPlayers: ThreadSafe + HandlesPlayer + PlayerMainCamera,
	TGraphics: ThreadSafe + PlayerCameras,
{
	fn build(&self, _: &mut App) {}
}
