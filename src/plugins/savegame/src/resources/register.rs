use crate::{
	context::{Handlers, SaveContext},
	errors::LockPoisonedError,
};
use bevy::prelude::*;
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct Register {
	handlers: Handlers,
}

impl Register {
	pub(crate) fn update_context(
		context: Arc<Mutex<SaveContext>>,
	) -> impl Fn(Res<Self>) -> Result<(), LockPoisonedError> {
		move |register| {
			let Ok(mut context) = context.lock() else {
				return Err(LockPoisonedError);
			};

			context.handlers = register.handlers.clone();
			Ok(())
		}
	}

	pub(crate) fn register_component<T>(&mut self)
	where
		T: Component + Serialize,
	{
		self.handlers.push(SaveContext::handle::<T>);
	}
}

#[cfg(test)]
mod test_registration {
	use super::*;

	#[derive(Component, Serialize)]
	struct _A;

	#[derive(Component, Serialize)]
	struct _B;

	#[test]
	fn register_component() {
		let mut context = Register::default();

		context.register_component::<_A>();

		assert_eq!(
			vec![SaveContext::handle::<_A> as usize],
			context
				.handlers
				.into_iter()
				.map(|h| h as usize)
				.collect::<Vec<_>>()
		)
	}

	#[test]
	fn register_components() {
		let mut register = Register::default();

		register.register_component::<_A>();
		register.register_component::<_B>();

		assert_eq!(
			vec![
				SaveContext::handle::<_A> as usize,
				SaveContext::handle::<_B> as usize
			],
			register
				.handlers
				.into_iter()
				.map(|h| h as usize)
				.collect::<Vec<_>>()
		)
	}
}

#[cfg(test)]
mod test_update_context {
	use super::*;
	use crate::{context::Buffer, writer::FileWriter};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::test_tools::utils::SingleThreadedApp;

	fn setup(handlers: Handlers) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(Register { handlers });

		app
	}

	#[test]
	fn update_context() -> Result<(), RunSystemError> {
		fn a(_: &mut Buffer, _: EntityRef) {}
		fn b(_: &mut Buffer, _: EntityRef) {}

		let mut app = setup(vec![a, b]);
		let context = Arc::new(Mutex::new(SaveContext::new(FileWriter::to_destination(""))));

		_ = app
			.world_mut()
			.run_system_once(Register::update_context(context.clone()))?;

		assert_eq!(
			vec![a, b],
			context.lock().expect("COULD NOT LOCK CONTEXT").handlers,
		);
		Ok(())
	}
}
