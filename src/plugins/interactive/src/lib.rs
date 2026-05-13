mod components;
mod system_params;

use crate::system_params::interactive::InteractiveMut;
use bevy::prelude::*;
use common::traits::handles_interactive::HandlesInteractive;

pub struct InteractivePlugin;

impl Plugin for InteractivePlugin {
	fn build(&self, _: &mut App) {}
}

impl HandlesInteractive for InteractivePlugin {
	type TInteractiveMut = InteractiveMut<'static, 'static>;
}
