use crate::{
	context::{Handlers, SaveContext},
	errors::LockPoisonedError,
};
use bevy::prelude::*;
use serde::Serialize;
use std::{
	any::TypeId,
	collections::HashSet,
	sync::{Arc, Mutex},
};

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct Register {
	registered_types: HashSet<TypeId>,
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

	pub(crate) fn register_component<TComponent, TDto>(&mut self)
	where
		TComponent: Component + Clone,
		TDto: From<TComponent> + Serialize,
	{
		let type_id = TypeId::of::<TComponent>();

		if self.registered_types.contains(&type_id) {
			return;
		}

		self.registered_types.insert(type_id);
		self.handlers.push(SaveContext::handle::<TComponent, TDto>);
	}
}

#[cfg(test)]
mod test_registration {
	use super::*;

	#[derive(Component, Serialize, Clone)]
	struct _A;

	#[derive(Component, Serialize, Clone)]
	struct _B;

	#[test]
	fn register_component() {
		let mut context = Register::default();

		context.register_component::<_A, _A>();

		assert_eq!(
			vec![SaveContext::handle::<_A, _A> as usize],
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

		register.register_component::<_A, _A>();
		register.register_component::<_B, _B>();

		assert_eq!(
			vec![
				SaveContext::handle::<_A, _A> as usize,
				SaveContext::handle::<_B, _B> as usize
			],
			register
				.handlers
				.into_iter()
				.map(|h| h as usize)
				.collect::<Vec<_>>()
		)
	}

	#[test]
	fn register_components_only_once() {
		let mut register = Register::default();

		register.register_component::<_A, _A>();
		register.register_component::<_A, _A>();

		assert_eq!(
			vec![SaveContext::handle::<_A, _A> as usize],
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
	use crate::{context::SaveBuffer, file_io::FileIO};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::test_tools::utils::SingleThreadedApp;
	use serde_json::Error;
	use std::path::PathBuf;

	fn setup(handlers: Handlers) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(Register {
			handlers,
			..default()
		});

		app
	}

	#[test]
	fn update_context() -> Result<(), RunSystemError> {
		fn a(_: &mut SaveBuffer, _: EntityRef) -> Result<(), Error> {
			Ok(())
		}
		fn b(_: &mut SaveBuffer, _: EntityRef) -> Result<(), Error> {
			Err(serde::de::Error::custom("Let me break everything"))
		}

		let mut app = setup(vec![a, b]);
		let context = Arc::new(Mutex::new(SaveContext::from(FileIO::with_file(
			PathBuf::new(),
		))));

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
