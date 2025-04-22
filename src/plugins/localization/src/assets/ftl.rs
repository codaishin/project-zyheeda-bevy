pub(crate) mod loader;

use bevy::prelude::*;

#[derive(Asset, TypePath, Debug, PartialEq, Clone)]
pub struct Ftl(pub(crate) String);
