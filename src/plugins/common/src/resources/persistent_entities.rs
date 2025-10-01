use crate::components::persistent_entity::PersistentEntity;
use bevy::prelude::*;
use std::{collections::HashMap, fmt::Display};
use zyheeda_core::logger::{Log, Logger};

const ENTITY_ERROR: &str = "Entity not found";

#[derive(Resource, Debug, PartialEq, Default)]
pub struct PersistentEntities<TLogger = Logger>
where
	TLogger: Log,
{
	pub(crate) entities: HashMap<PersistentEntity, Entity>,
	pub(crate) logger: TLogger,
}

impl<TLogger> PersistentEntities<TLogger>
where
	TLogger: Log,
{
	pub(crate) fn get_entity(&self, persistent_entity: &PersistentEntity) -> Option<Entity> {
		let Some(entity) = self.entities.get(persistent_entity) else {
			self.logger
				.log_warning(ENTITY_ERROR, NoMatch(*persistent_entity));
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

#[cfg(test)]
mod tests {
	use super::*;
	use macros::simple_mock;
	use mockall::predicate::eq;
	use testing::Mock;

	simple_mock! {
		_Logger {}
		impl Log for _Logger {
			fn log_warning<TContext>(&self, label: &str, context: TContext) where TContext: 'static;
			fn log_error<TContext>(&self, label: &str, context: TContext) where TContext: 'static;
		}
	}

	#[test]
	fn get_entity() {
		let target = Entity::from_raw(42);
		let persistent_entity = PersistentEntity::default();
		let persistent_entities = PersistentEntities {
			entities: HashMap::from([(persistent_entity, target)]),
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_error::<NoMatch>().never();
			}),
		};

		let entity = persistent_entities.get_entity(&persistent_entity);

		assert_eq!(Some(target), entity);
	}

	#[test]
	fn log_misses() {
		let persistent_entity = PersistentEntity::default();
		let persistent_entities = PersistentEntities {
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_warning()
					.times(1)
					.with(eq(ENTITY_ERROR), eq(NoMatch(persistent_entity)))
					.return_const(());
			}),
			..default()
		};

		persistent_entities.get_entity(&persistent_entity);
	}
}
