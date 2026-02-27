pub(crate) mod grid_min;
pub(crate) mod key_mapper;
pub(crate) mod to_subdivided;

use bevy::prelude::*;
use common::{
	traits::{handles_lights::HandlesLights, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaEntityCommands,
};

pub(crate) trait ExtraComponentsDefinition {
	fn target_names() -> Vec<String>;
	fn insert_bundle<TLights>(entity: &mut ZyheedaEntityCommands)
	where
		TLights: HandlesLights + ThreadSafe;
}
