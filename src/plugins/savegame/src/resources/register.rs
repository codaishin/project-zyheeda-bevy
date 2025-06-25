use crate::{
	context::{SaveContext, handler::ComponentHandler},
	errors::LockPoisonedError,
	file_io::FileIO,
};
use bevy::prelude::*;
use common::traits::{
	handles_saving::SavableComponent,
	load_asset::LoadAsset,
	thread_safe::ThreadSafe,
};
use std::{
	any::TypeId,
	collections::HashSet,
	sync::{Arc, Mutex},
};

#[derive(Resource, Debug)]
pub(crate) struct Register<TLoadAsset = AssetServer> {
	registered_types: HashSet<TypeId>,
	priority_handlers: Vec<ComponentHandler<TLoadAsset>>,
	handlers: Vec<ComponentHandler<TLoadAsset>>,
}

impl<TLoadAsset> Register<TLoadAsset>
where
	TLoadAsset: ThreadSafe + Clone + LoadAsset,
{
	pub(crate) fn update_context(
		context: Arc<Mutex<SaveContext<FileIO, ComponentHandler<TLoadAsset>>>>,
	) -> impl Fn(Res<Self>) -> Result<(), LockPoisonedError> {
		move |register| {
			let Ok(mut context) = context.lock() else {
				return Err(LockPoisonedError);
			};

			context.handlers = register.handlers.clone();
			context.priority_handlers = register.priority_handlers.clone();
			Ok(())
		}
	}

	pub(crate) fn register_component<TComponent>(&mut self)
	where
		TComponent: SavableComponent,
	{
		let type_id = TypeId::of::<TComponent>();

		if self.registered_types.contains(&type_id) {
			return;
		}

		self.registered_types.insert(type_id);

		let handlers = match TComponent::PRIORITY {
			true => &mut self.priority_handlers,
			false => &mut self.handlers,
		};

		handlers.push(ComponentHandler::new::<TComponent>());
	}
}

impl<TLoadAsset> Default for Register<TLoadAsset>
where
	TLoadAsset: LoadAsset,
{
	fn default() -> Self {
		Self {
			registered_types: HashSet::default(),
			priority_handlers: vec![],
			handlers: vec![],
		}
	}
}

impl<TLoadAsset> PartialEq for Register<TLoadAsset> {
	fn eq(&self, other: &Self) -> bool {
		self.registered_types == other.registered_types && self.handlers == other.handlers
	}
}

#[cfg(test)]
mod test_registration {
	use super::*;
	use common::impl_savable_self_non_priority;
	use serde::{Deserialize, Serialize};

	#[derive(Component, Serialize, Deserialize, Clone)]
	struct _A;

	#[derive(Component, Serialize, Deserialize, Clone)]
	struct _B;

	impl_savable_self_non_priority!(_A, _B);

	#[derive(Component, Serialize, Deserialize, Clone)]
	struct _PA;

	impl SavableComponent for _PA {
		type TDto = Self;
		const PRIORITY: bool = true;
	}

	#[derive(Component, Serialize, Deserialize, Clone)]
	struct _PB;

	impl SavableComponent for _PB {
		type TDto = Self;
		const PRIORITY: bool = true;
	}

	#[test]
	fn register_component() {
		let mut register = Register::<AssetServer>::default();

		register.register_component::<_A>();

		assert_eq!(vec![ComponentHandler::new::<_A>()], register.handlers);
	}

	#[test]
	fn register_components() {
		let mut register = Register::<AssetServer>::default();

		register.register_component::<_A>();
		register.register_component::<_B>();

		assert_eq!(
			(
				vec![],
				vec![ComponentHandler::new::<_A>(), ComponentHandler::new::<_B>()],
			),
			(register.priority_handlers, register.handlers,)
		);
	}

	#[test]
	fn register_priority_components() {
		let mut register = Register::<AssetServer>::default();

		register.register_component::<_PA>();
		register.register_component::<_PB>();

		assert_eq!(
			(
				vec![
					ComponentHandler::new::<_PA>(),
					ComponentHandler::new::<_PB>()
				],
				vec![],
			),
			(register.priority_handlers, register.handlers,)
		);
	}

	#[test]
	fn register_components_only_once() {
		let mut register = Register::<AssetServer>::default();

		register.register_component::<_A>();
		register.register_component::<_A>();
		register.register_component::<_PA>();
		register.register_component::<_PA>();

		assert_eq!(
			(
				vec![ComponentHandler::new::<_PA>()],
				vec![ComponentHandler::new::<_A>()],
			),
			(register.priority_handlers, register.handlers,)
		);
	}
}

#[cfg(test)]
mod test_update_context {
	use super::*;
	use crate::file_io::FileIO;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{impl_savable_self_non_priority, test_tools::utils::SingleThreadedApp};
	use serde::{Deserialize, Serialize};
	use std::{ops::Deref, path::PathBuf};

	#[derive(Component, Serialize, Deserialize, Clone)]
	struct _A;

	#[derive(Component, Serialize, Deserialize, Clone)]
	struct _B;

	#[derive(Component, Serialize, Deserialize, Clone)]
	struct _C;

	#[derive(Component, Serialize, Deserialize, Clone)]
	struct _D;

	fn setup(handlers: Vec<ComponentHandler>, priority_handlers: Vec<ComponentHandler>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(Register {
			handlers,
			priority_handlers,
			..default()
		});

		app
	}

	impl_savable_self_non_priority!(_A, _B, _C, _D);

	#[test]
	fn update_context() -> Result<(), RunSystemError> {
		let handlers = vec![ComponentHandler::new::<_A>(), ComponentHandler::new::<_B>()];
		let priority = vec![ComponentHandler::new::<_C>(), ComponentHandler::new::<_D>()];
		let mut app = setup(handlers.clone(), priority.clone());
		let context = Arc::new(Mutex::new(SaveContext::from(FileIO::with_file(
			PathBuf::new(),
		))));

		_ = app
			.world_mut()
			.run_system_once(Register::update_context(context.clone()))?;

		assert_eq!(
			&SaveContext::from(FileIO::with_file(PathBuf::new()))
				.with_handlers(handlers)
				.with_priority_handlers(priority),
			context.lock().expect("COULD NOT LOCK CONTEXT").deref(),
		);
		Ok(())
	}
}
