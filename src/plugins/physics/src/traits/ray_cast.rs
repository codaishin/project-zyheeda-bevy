mod ground;
mod solid_objects;

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_rapier3d::prelude::*;

/// A simple wrapper around rapier's [`ReadRapierContext`], so we can implement
/// external traits for it.
#[derive(SystemParam)]
pub struct RayCaster<'w, 's> {
	context: ReadRapierContext<'w, 's>,
}
