use bevy::prelude::*;
use common::traits::handles_game_states::GameState;

#[derive(Event, Debug, PartialEq, Clone, Copy)]
pub(crate) struct GameStateEvent(pub(crate) GameState);
