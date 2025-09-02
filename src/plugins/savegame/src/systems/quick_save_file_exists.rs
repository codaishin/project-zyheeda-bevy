use crate::{
	errors::LockPoisonedError,
	resources::inspector::Inspector,
	traits::file_exists::FileExists,
};
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;

impl<TIO> Inspector<TIO>
where
	TIO: FileExists + ThreadSafe,
{
	pub(crate) fn quick_save_file_exists(inspector: Res<Self>) -> Result<bool, LockPoisonedError> {
		let Ok(quick_save) = inspector.quick_save.lock() else {
			return Err(LockPoisonedError);
		};

		Ok(quick_save.io.file_exists())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::context::SaveContext;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use macros::simple_mock;
	use std::sync::{Arc, Mutex};
	use testing::{Mock, SingleThreadedApp};

	simple_mock! {
		_IO {}
		impl FileExists for _IO {
			fn file_exists(&self) -> bool;
		}
	}

	fn setup(quick_save: SaveContext<Mock_IO>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(Inspector {
			quick_save: Arc::new(Mutex::new(quick_save)),
		});

		app
	}

	#[test]
	fn file_exists() -> Result<(), RunSystemError> {
		let mut app = setup(SaveContext::from(Mock_IO::new_mock(|mock| {
			mock.expect_file_exists().return_const(true);
		})));

		let result = app
			.world_mut()
			.run_system_once(Inspector::<Mock_IO>::quick_save_file_exists)?;

		assert_eq!(Ok(true), result);
		Ok(())
	}

	#[test]
	fn file_does_not_exist() -> Result<(), RunSystemError> {
		let mut app = setup(SaveContext::from(Mock_IO::new_mock(|mock| {
			mock.expect_file_exists().return_const(false);
		})));

		let result = app
			.world_mut()
			.run_system_once(Inspector::<Mock_IO>::quick_save_file_exists)?;

		assert_eq!(Ok(false), result);
		Ok(())
	}
}
