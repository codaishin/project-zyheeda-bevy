use crate::traits::get_errors_mut::GetErrorsMut;
use bevy::prelude::*;
use common::traits::or_ok::OrOk;

impl<T> DrainErrors for T where T: GetErrorsMut + Resource {}

pub(crate) trait DrainErrors: GetErrorsMut + Resource {
	fn drain_errors(mut ftl_server: ResMut<Self>) -> Result<(), Vec<Self::TError>> {
		ftl_server
			.errors_mut()
			.drain(..)
			.collect::<Vec<_>>()
			.or_ok(|| ())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	#[derive(Resource)]
	struct _FtlServer(Vec<_Error>);

	impl GetErrorsMut for _FtlServer {
		type TError = _Error;

		fn errors_mut(&mut self) -> &mut Vec<Self::TError> {
			&mut self.0
		}
	}

	#[derive(Debug, PartialEq)]
	struct _Error;

	fn setup(ftl_server: _FtlServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(ftl_server);

		app
	}

	#[test]
	fn remove_errors() -> Result<(), RunSystemError> {
		let mut app = setup(_FtlServer(vec![_Error, _Error]));

		_ = app.world_mut().run_system_once(_FtlServer::drain_errors)?;

		let server = app.world().resource::<_FtlServer>();
		assert!(server.0.is_empty());
		Ok(())
	}

	#[test]
	fn return_errors() -> Result<(), RunSystemError> {
		let mut app = setup(_FtlServer(vec![_Error, _Error]));

		let errors = app.world_mut().run_system_once(_FtlServer::drain_errors)?;

		assert_eq!(Err(vec![_Error, _Error]), errors);
		Ok(())
	}
}
