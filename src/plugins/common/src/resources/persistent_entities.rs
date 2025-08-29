use crate::components::persistent_entity::PersistentEntity;
use bevy::prelude::*;
use std::collections::HashMap;
use zyheeda_core::logger::{Log, Logger};

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
				.log_warning(format!("{persistent_entity:?}: no matching entity found"));
			return None;
		};

		Some(*entity)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use mockall::{mock, predicate::eq};
	use testing::{Mock, simple_init};

	mock! {
		_Logger {}
		impl Log for _Logger {
			fn log_warning<TError>(&self, value: TError) where TError: Into<String> + 'static;
			fn log_error<TError>(&self, value: TError) where TError: Into<String> + 'static;
		}
	}

	simple_init!(Mock_Logger);

	#[test]
	fn get_entity() {
		let target = Entity::from_raw(42);
		let persistent_entity = PersistentEntity::default();
		let persistent_entities = PersistentEntities {
			entities: HashMap::from([(persistent_entity, target)]),
			logger: Mock_Logger::new(),
		};

		let entity = persistent_entities.get_entity(&persistent_entity);

		assert_eq!(Some(target), entity);
	}

	#[test]
	fn log_misses() {
		let persistent_entity = PersistentEntity::default();
		let persistent_entities = PersistentEntities {
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_warning::<String>()
					.times(1)
					.with(eq(format!(
						"{persistent_entity:?}: no matching entity found"
					)))
					.return_const(());
			}),
			..default()
		};

		persistent_entities.get_entity(&persistent_entity);
	}
}
