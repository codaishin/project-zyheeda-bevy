use crate::components::player::Player;
use bevy::{
	ecs::{query::QuerySingleError, system::SystemParam},
	prelude::*,
};
use common::{
	components::persistent_entity::PersistentEntity,
	error_logger::{GlobalErrorLogger, Log},
	errors::{ErrorData, Level},
	traits::{accessors::get::View, handles_player::PlayerEntity},
};
use std::{fmt::Display, time::Duration};

#[derive(SystemParam)]
pub struct PlayerParam<'w, 's> {
	players: Query<'w, 's, &'static PersistentEntity, With<Player>>,
}

impl View<PlayerEntity> for PlayerParam<'_, '_> {
	fn view(&self) -> Option<PersistentEntity> {
		match self.players.single() {
			Ok(player) => Some(*player),
			Err(QuerySingleError::NoEntities(_)) => {
				GlobalErrorLogger::INSTANCE.log(NoPlayerError);
				None
			}
			Err(QuerySingleError::MultipleEntities(msg)) => {
				GlobalErrorLogger::INSTANCE.log(MultiplePlayersError(msg));
				None
			}
		}
	}
}

#[derive(Debug)]
struct NoPlayerError;

impl Display for NoPlayerError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "no player found")
	}
}

impl ErrorData for NoPlayerError {
	fn rate_limit() -> Option<Duration> {
		Some(Duration::from_secs(1))
	}

	fn level(&self) -> Level {
		Level::Warning
	}

	fn label() -> impl Display {
		"No Player Error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[derive(Debug)]
struct MultiplePlayersError(DebugName);

impl Display for MultiplePlayersError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "multiple players found: {}", self.0)
	}
}

impl ErrorData for MultiplePlayersError {
	fn rate_limit() -> Option<Duration> {
		Some(Duration::from_secs(1))
	}

	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Multiple Players Error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}
