use bevy::prelude::*;
use common::prelude::*;
use std::collections::HashSet;

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct ConfiguredTransitions(pub(crate) HashSet<ActivityState>);
