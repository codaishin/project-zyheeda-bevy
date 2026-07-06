use crate::components::{agent::Agent, player::Player};
use bevy::{
	ecs::{
		query::{QueryFilter, QueryIter, QuerySingleError},
		system::{StaticSystemParam, SystemParam},
	},
	prelude::*,
};
use common::{
	components::persistent_entity::PersistentEntity,
	error_logger::{GlobalErrorLogger, Log},
	errors::{ErrorData, Level},
	traits::{accessors::get::View, handles_player::PlayerEntity},
};
use std::{fmt::Display, iter::Copied, time::Duration};

#[derive(SystemParam)]
pub struct AgentParam<'w, 's, TFilter, TAgent = Agent, TLogger = GlobalErrorLogger>
where
	TFilter: QueryFilter + 'static,
	TAgent: Component,
	TLogger: for<'w2, 's2> SystemParam<Item<'w2, 's2>: Log> + 'static,
{
	agents: Query<'w, 's, &'static PersistentEntity, (With<TAgent>, TFilter)>,
	logger: StaticSystemParam<'w, 's, TLogger>,
}

impl<'w, 's, TFilter, TAgent> IntoIterator for AgentParam<'w, 's, TFilter, TAgent>
where
	TFilter: QueryFilter + 'static,
	TAgent: Component,
{
	type Item = PersistentEntity;
	type IntoIter = Copied<QueryIter<'w, 's, &'static PersistentEntity, (With<TAgent>, TFilter)>>;

	fn into_iter(self) -> Self::IntoIter {
		self.agents.into_iter().copied()
	}
}

impl<TLogger> View<PlayerEntity> for AgentParam<'_, '_, (), Player, TLogger>
where
	TLogger: for<'w2, 's2> SystemParam<Item<'w2, 's2>: Log> + 'static,
{
	fn view(&self) -> Option<PersistentEntity> {
		match self.agents.single() {
			Ok(player) => Some(*player),
			Err(QuerySingleError::NoEntities(_)) => {
				self.logger.log(NoPlayerError);
				None
			}
			Err(QuerySingleError::MultipleEntities(msg)) => {
				self.logger.log(MultiplePlayersError(msg));
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

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use std::{any::TypeId, sync::RwLock};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	#[derive(Resource)]
	struct _Logger {
		errors: RwLock<Vec<TypeId>>,
	}

	impl Log for _Logger {
		fn log<TError>(&self, _: TError)
		where
			TError: ErrorData,
		{
			let Ok(mut lock) = self.errors.write() else {
				return;
			};

			lock.push(TypeId::of::<TError>());
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(_Logger {
			errors: RwLock::new(vec![]),
		});

		app
	}

	#[test]
	fn return_entities() -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = PersistentEntity::default();
		let b = PersistentEntity::default();
		app.world_mut().spawn((a, _Agent));
		app.world_mut().spawn((b, _Agent));

		let entities = app
			.world_mut()
			.run_system_once(|a: AgentParam<(), _Agent>| a.into_iter().collect::<Vec<_>>())?;

		assert_eq!(vec![a, b], entities);
		Ok(())
	}

	#[test]
	fn skip_non_agents() -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = PersistentEntity::default();
		let b = PersistentEntity::default();
		let c = PersistentEntity::default();
		app.world_mut().spawn((a, _Agent));
		app.world_mut().spawn(b);
		app.world_mut().spawn((c, _Agent));

		let entities = app
			.world_mut()
			.run_system_once(|a: AgentParam<(), _Agent>| a.into_iter().collect::<Vec<_>>())?;

		assert_eq!(vec![a, c], entities);
		Ok(())
	}

	#[test]
	fn skip_mismatching_filter() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct Skip;

		let mut app = setup();
		let a = PersistentEntity::default();
		let b = PersistentEntity::default();
		let c = PersistentEntity::default();
		app.world_mut().spawn((a, _Agent));
		app.world_mut().spawn((b, Skip, _Agent));
		app.world_mut().spawn((c, _Agent));

		let entities =
			app.world_mut()
				.run_system_once(|a: AgentParam<Without<Skip>, _Agent>| {
					a.into_iter().collect::<Vec<_>>()
				})?;

		assert_eq!(vec![a, c], entities);
		Ok(())
	}

	#[test]
	fn player_view() -> Result<(), RunSystemError> {
		let mut app = setup();
		let player = PersistentEntity::default();
		app.world_mut().spawn((player, Player));

		let view = app
			.world_mut()
			.run_system_once(|a: AgentParam<(), Player>| a.view())?;

		assert_eq!(Some(player), view);
		Ok(())
	}

	#[test]
	fn log_no_player() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once(|a: AgentParam<(), Player, Res<_Logger>>| a.view())?;

		assert_eq!(
			vec![TypeId::of::<NoPlayerError>()],
			*app.world().resource::<_Logger>().errors.read().unwrap(),
		);
		Ok(())
	}

	#[test]
	fn log_multiple_players_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((PersistentEntity::default(), Player));
		app.world_mut().spawn((PersistentEntity::default(), Player));

		app.world_mut()
			.run_system_once(|a: AgentParam<(), Player, Res<_Logger>>| a.view())?;

		assert_eq!(
			vec![TypeId::of::<MultiplePlayersError>()],
			*app.world().resource::<_Logger>().errors.read().unwrap(),
		);
		Ok(())
	}
}
