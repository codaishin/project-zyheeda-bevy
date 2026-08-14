use crate::components::interactive::Interactive;
use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Interactive { interactive_type: InteractiveType::Container })]
pub(crate) struct Container;
