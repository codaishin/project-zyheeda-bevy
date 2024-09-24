pub mod components;

use bevy::prelude::*;
use common::systems::{
	move_component::MoveInto,
	remove_component::Remove,
	remove_component_from_children::RemoveFromChildren,
};
use components::effect_shader::EffectShaders;

pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(
				Handle::<StandardMaterial>::remove_from::<EffectShaders>,
				Handle::<StandardMaterial>::remove_from_children_of::<EffectShaders>,
				Handle::<Mesh>::move_into::<EffectShaders>,
			),
		);
	}
}
