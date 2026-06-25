use crate::components::camera_labels::{
	AgentsPass,
	CompositePass,
	EffectLightPass,
	OutlinePass,
	VisibilityPass,
	WorldLight,
	WorldPass,
};
use bevy::prelude::*;

pub(crate) fn spawn_cameras(mut commands: Commands) {
	commands.spawn(WorldPass);
	commands.spawn(AgentsPass);
	commands.spawn(VisibilityPass);
	commands.spawn(EffectLightPass);
	commands.spawn(OutlinePass);
	commands.spawn(CompositePass);
	commands.spawn(WorldLight);
}
