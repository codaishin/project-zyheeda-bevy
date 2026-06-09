use crate::{
	components::persistent_entity::PersistentEntity,
	error_logger::{ErrorLogger, Log},
	errors::{ErrorData, Level},
	traits::thread_safe::ThreadSafe,
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use std::{collections::HashMap, fmt::Display};

#[derive(Resource, Debug, PartialEq, Default)]
pub struct PersistentEntities(pub(crate) HashMap<PersistentEntity, Entity>);

#[derive(SystemParam)]
pub(crate) struct PersistentEntitiesParam<'w, 's, TLogger = ErrorLogger>
where
	TLogger: SystemParam + ThreadSafe,
{
	pub(crate) entities: Option<Res<'w, PersistentEntities>>,
	pub(crate) logger: StaticSystemParam<'w, 's, TLogger>,
}

impl<'w, 's, TLogger> PersistentEntitiesParam<'w, 's, TLogger>
where
	TLogger: for<'w2, 's2> SystemParam<Item<'w2, 's2>: Log> + ThreadSafe,
{
	pub(crate) fn get_entity(&self, persistent_entity: &PersistentEntity) -> Option<Entity> {
		let entities = self.entities.as_ref()?;
		let Some(entity) = entities.0.get(persistent_entity) else {
			self.logger.log(NoMatch(*persistent_entity));
			return None;
		};

		Some(*entity)
	}
}

#[derive(Debug, PartialEq)]
struct NoMatch(PersistentEntity);

impl Display for NoMatch {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let Self(persistent_entity) = self;
		write!(f, "{persistent_entity:?}: no matching entity found")
	}
}

impl ErrorData for NoMatch {
	fn level(&self) -> crate::errors::Level {
		Level::Warning
	}

	fn label() -> impl Display {
		"Entity not found"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, fake_entity};

	#[derive(Resource, NestedMocks)]
	struct _Logger {
		mock: Mock_Logger,
	}

	#[automock]
	impl Log for _Logger {
		fn log<TError>(&self, error: TError)
		where
			TError: ErrorData + 'static,
		{
			self.mock.log(error)
		}
	}

	fn setup(entities: impl Into<HashMap<PersistentEntity, Entity>>, logger: _Logger) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(PersistentEntities(entities.into()));
		app.insert_resource(logger);

		app
	}

	#[test]
	fn get_entity() -> Result<(), RunSystemError> {
		let target = fake_entity!(42);
		let persistent_entity = PersistentEntity::default();
		let mut app = setup(
			[(persistent_entity, target)],
			_Logger::new().with_mock(|mock| {
				mock.expect_log::<NoMatch>().never();
			}),
		);

		let entity = app.world_mut().run_system_once(
			move |p: PersistentEntitiesParam<Res<'static, _Logger>>| {
				p.get_entity(&persistent_entity)
			},
		)?;

		assert_eq!(Some(target), entity);
		Ok(())
	}

	#[test]
	fn log_misses() -> Result<(), RunSystemError> {
		let persistent_entity = PersistentEntity::default();
		let mut app = setup(
			[],
			_Logger::new().with_mock(|mock| {
				mock.expect_log()
					.once()
					.with(eq(NoMatch(persistent_entity)))
					.return_const(());
			}),
		);

		app.world_mut()
			.run_system_once(move |p: PersistentEntitiesParam<Res<'static, _Logger>>| {
				p.get_entity(&persistent_entity);
			})
	}
}
