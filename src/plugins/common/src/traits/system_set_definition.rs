use crate::tools::plugin_system_set::PluginSystemSet;
use bevy::prelude::*;

pub trait SystemSetDefinition {
	type TSystemSet: SystemSet;

	const SYSTEMS: PluginSystemSet<Self::TSystemSet>;
}
