use crate::{
	components::save::Save,
	errors::{LockPoisonedError, SerializationErrors, SerializationOrLockError},
	traits::execute_save::BufferComponents,
};
use bevy::prelude::*;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

impl<T> BufferSystem for T {}

pub trait BufferSystem {
	fn buffer_system(
		context: Arc<Mutex<Self>>,
	) -> impl Fn(&mut World) -> Result<(), SerializationOrLockError>
	where
		Self: BufferComponents,
	{
		move |world| {
			let Ok(mut context) = context.lock() else {
				return Err(SerializationOrLockError::LockPoisoned(LockPoisonedError));
			};

			let errors = world
				.iter_entities()
				.filter(|entity| entity.contains::<Save>())
				.filter_map(|entity| match context.buffer_components(entity) {
					Ok(()) => None,
					Err(errors) => Some((entity.id(), errors)),
				})
				.collect::<HashMap<_, _>>();

			match errors.is_empty() {
				true => Ok(()),
				false => Err(SerializationOrLockError::SerializationErrors(
					SerializationErrors(errors),
				)),
			}
		}
	}
}

#[cfg(test)]
mod test_save {
	use super::*;
	use crate::errors::EntitySerializationErrors;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{simple_init, test_tools::utils::SingleThreadedApp, traits::mock::Mock};
	use mockall::mock;

	mock! {
		_SaveContext {}
		impl BufferComponents for _SaveContext {
			fn buffer_components<'a>(&mut self, entity: EntityRef<'a>) -> Result<(), EntitySerializationErrors>;
		}
	}

	simple_init!(Mock_SaveContext);

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn buffer() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(Save).id();
		let context = Mock_SaveContext::new_mock(|mock| {
			mock.expect_buffer_components()
				.times(1)
				.withf(move |entity_ref| entity_ref.id() == entity)
				.returning(|_| Ok(()));
		});
		let context = Arc::new(Mutex::new(context));

		_ = app
			.world_mut()
			.run_system_once(Mock_SaveContext::buffer_system(context))?;
		Ok(())
	}

	#[test]
	fn ignore_entities_without_save() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let context = Mock_SaveContext::new_mock(|mock| {
			mock.expect_buffer_components()
				.never()
				.withf(move |entity_ref| entity_ref.id() == entity)
				.returning(|_| Ok(()));
		});
		let context = Arc::new(Mutex::new(context));

		_ = app
			.world_mut()
			.run_system_once(Mock_SaveContext::buffer_system(context))?;
		Ok(())
	}

	#[test]
	fn serialization_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = app.world_mut().spawn(Save).id();
		let b = app.world_mut().spawn(Save).id();
		let context = Mock_SaveContext::new_mock(|mock| {
			mock.expect_buffer_components().returning(|_| {
				Err(EntitySerializationErrors(vec![
					serde::ser::Error::custom("that"),
					serde::ser::Error::custom("failed"),
				]))
			});
		});
		let context = Arc::new(Mutex::new(context));

		let result = app
			.world_mut()
			.run_system_once(Mock_SaveContext::buffer_system(context))?;

		assert_eq!(
			Err(HashMap::from([
				(a, "that failed".to_owned()),
				(b, "that failed".to_owned()),
			])),
			// that hurts, but of course serde's errors are not comparable...,
			// so we convert the errors to strings
			result.map_err(|e| match e {
				SerializationOrLockError::LockPoisoned(_) => HashMap::default(),
				SerializationOrLockError::SerializationErrors(SerializationErrors(errors)) =>
					errors
						.iter()
						.map(|(entity, EntitySerializationErrors(errors))| (
							*entity,
							errors
								.iter()
								.map(|error| error.to_string())
								.collect::<Vec<_>>()
								.join(" ")
						))
						.collect::<HashMap<_, _>>(),
			})
		);
		Ok(())
	}
}
