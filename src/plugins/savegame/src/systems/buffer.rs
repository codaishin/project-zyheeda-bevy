use crate::{
	components::save::Save,
	errors::LockPoisonedError,
	traits::execute_save::BufferComponents,
};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

impl<T> BufferSystem for T {}

pub trait BufferSystem {
	fn buffer_system(
		context: Arc<Mutex<Self>>,
	) -> impl Fn(&mut World) -> Result<(), LockPoisonedError>
	where
		Self: BufferComponents,
	{
		move |world| {
			let Ok(mut context) = context.lock() else {
				return Err(LockPoisonedError);
			};
			let entities = world
				.iter_entities()
				.filter(|entity| entity.contains::<Save>());

			for entity in entities {
				context.buffer_components(entity);
			}

			Ok(())
		}
	}
}

#[cfg(test)]
mod test_save {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{simple_init, test_tools::utils::SingleThreadedApp, traits::mock::Mock};
	use mockall::mock;

	mock! {
		_SaveContext {}
		impl BufferComponents for _SaveContext {
			fn buffer_components<'a>(&mut self, entity: EntityRef<'a>);
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
				.return_const(());
		});
		let context = Arc::new(Mutex::new(context));

		_ = app
			.world_mut()
			.run_system_once(Mock_SaveContext::buffer_system(context))?;
		Ok(())
	}
}
