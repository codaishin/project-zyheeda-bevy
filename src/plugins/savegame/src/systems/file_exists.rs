use crate::{context::SaveContext, errors::LockPoisonedError, traits::file_exists::FileExists};
use std::sync::{Arc, Mutex};

impl<TIO> SaveContext<TIO>
where
	TIO: FileExists,
{
	pub(crate) fn file_exists(
		context: Arc<Mutex<Self>>,
	) -> impl Fn() -> Result<bool, LockPoisonedError> {
		move || {
			let Ok(context) = context.lock() else {
				return Err(LockPoisonedError);
			};

			Ok(context.io.file_exists())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use mockall::mock;
	use testing::{Mock, SingleThreadedApp, simple_init};

	mock! {
		_IO {}
		impl FileExists for _IO {
			fn file_exists(&self) -> bool;
		}
	}

	simple_init!(Mock_IO);

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn file_exists() -> Result<(), RunSystemError> {
		let mut app = setup();
		let ctx = Arc::new(Mutex::new(SaveContext::from(Mock_IO::new_mock(|mock| {
			mock.expect_file_exists().return_const(true);
		}))));

		assert_eq!(
			Ok(true),
			app.world_mut()
				.run_system_once(SaveContext::file_exists(ctx))?
		);
		Ok(())
	}

	#[test]
	fn file_does_not_exist() -> Result<(), RunSystemError> {
		let mut app = setup();
		let ctx = Arc::new(Mutex::new(SaveContext::from(Mock_IO::new_mock(|mock| {
			mock.expect_file_exists().return_const(false);
		}))));

		assert_eq!(
			Ok(false),
			app.world_mut()
				.run_system_once(SaveContext::file_exists(ctx))?
		);
		Ok(())
	}
}
