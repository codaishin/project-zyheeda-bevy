use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct Player;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct Enemy;
