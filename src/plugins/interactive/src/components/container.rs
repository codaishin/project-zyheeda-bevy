use crate::components::interactive::Interactive;
use bevy::prelude::*;
use common::traits::handles_map_generation::InteractiveType;

#[derive(Component, Debug, PartialEq)]
#[require(Interactive { interactive_type: InteractiveType::Container })]
pub(crate) struct Container;
