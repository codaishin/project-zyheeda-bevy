use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct OngoingInteractions(pub(crate) HashMap<Entity, HashSet<Entity>>);
