use std::sync::{Arc, Mutex};

use crate::{context::SaveContext, traits::execute_save::ExecuteSave};
use bevy::prelude::*;
use serde::Serialize;

#[derive(Component, Debug, PartialEq, Default)]
pub struct Save<TSaveContext = SaveContext>
where
	TSaveContext: 'static,
{
	handlers: Vec<fn(&mut TSaveContext, EntityRef)>,
}

impl<TSaveContext> Save<TSaveContext>
where
	TSaveContext: ExecuteSave,
{
	pub fn handling<TComponent>(mut self) -> Self
	where
		TComponent: Component + Serialize + 'static,
	{
		self.handlers.push(TSaveContext::execute_save::<TComponent>);

		self
	}

	pub fn save_system_via(context: Arc<Mutex<TSaveContext>>) -> impl Fn(&mut World) {
		move |world| {
			let Ok(mut context) = context.try_lock() else {
				return;
			};
			let entities = world
				.iter_entities()
				.filter_map(|entity| Some((entity, entity.get::<Self>()?)));

			for (entity, Save { handlers }) in entities {
				for handler in handlers {
					handler(&mut context, entity);
				}
			}
		}
	}
}

#[cfg(test)]
mod test_adding_handlers {
	use super::*;

	#[derive(Component, Serialize)]
	struct _A;

	#[derive(Component, Serialize)]
	struct _B;

	#[test]
	fn store_save_fn() {
		let save = Save::default();

		let save = save.handling::<_A>();

		assert_eq!(
			Save {
				handlers: vec![SaveContext::execute_save::<_A>]
			},
			save
		);
	}

	#[test]
	fn store_save_fns() {
		let save = Save::default();

		let save = save.handling::<_A>().handling::<_B>();

		assert_eq!(
			Save {
				handlers: vec![
					SaveContext::execute_save::<_A>,
					SaveContext::execute_save::<_B>
				]
			},
			save
		);
	}
}

#[cfg(test)]
mod test_save {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;
	use std::any::TypeId;

	#[derive(Default)]
	struct _SaveContext {
		called_with: Vec<(Entity, TypeId)>,
	}

	impl ExecuteSave for _SaveContext {
		fn execute_save<T>(&mut self, entity: EntityRef)
		where
			T: 'static,
		{
			self.called_with.push((entity.id(), TypeId::of::<T>()));
		}
	}

	#[derive(Component, Serialize)]
	struct _A;

	#[derive(Component, Serialize)]
	struct _B;

	fn setup(context: Arc<Mutex<_SaveContext>>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, Save::save_system_via(context));

		app
	}

	#[test]
	fn execute_save() {
		let context = Arc::new(Mutex::new(_SaveContext::default()));
		let mut app = setup(context.clone());
		let entity = app
			.world_mut()
			.spawn(
				Save::<_SaveContext>::default()
					.handling::<_A>()
					.handling::<_B>(),
			)
			.id();

		app.update();

		assert_eq!(
			vec![(entity, TypeId::of::<_A>()), (entity, TypeId::of::<_B>())],
			context.lock().expect("COULD NOT LOCK CONTEXT").called_with
		);
	}
}
