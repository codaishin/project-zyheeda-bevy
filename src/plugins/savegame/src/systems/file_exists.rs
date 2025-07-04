use crate::{context::SaveContext, traits::file_exists::FileExists};
use std::sync::{Arc, Mutex};

impl<TIO> SaveContext<TIO>
where
	TIO: FileExists,
{
	pub(crate) fn file_exists(context: Arc<Mutex<Self>>) -> impl Fn() -> bool {
		move || {
			let Ok(context) = context.lock() else {
				return false;
			};

			context.io.file_exists()
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
	use common::{simple_init, test_tools::utils::SingleThreadedApp, traits::mock::Mock};
	use mockall::mock;

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

		assert!(
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

		assert!(
			!app.world_mut()
				.run_system_once(SaveContext::file_exists(ctx))?
		);
		Ok(())
	}
}
