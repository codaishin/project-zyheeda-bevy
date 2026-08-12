use bevy::prelude::*;
use common::traits::handles_game_states::ActivityState;
use std::collections::HashSet;

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct ConfiguredTransitions(pub(crate) HashSet<ActivityState>);
