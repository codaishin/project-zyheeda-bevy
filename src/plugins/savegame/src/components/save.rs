use crate::{context::SaveContext, traits::execute_save::ExecuteSave, writer::FileWriter};
use bevy::prelude::*;
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Component, Debug, PartialEq)]
pub struct Save<TSaveContext = SaveContext<FileWriter>>
where
	TSaveContext: 'static,
{
	handlers: Vec<fn(&mut TSaveContext, EntityRef)>,
}

impl<TSaveContext> Save<TSaveContext>
where
	TSaveContext: ExecuteSave,
{
	pub fn component<TComponent>() -> Self
	where
		TComponent: Component + Serialize + 'static,
	{
		Self {
			handlers: vec![TSaveContext::buffer::<TComponent>],
		}
	}

	pub fn and_component<TComponent>(mut self) -> Self
	where
		TComponent: Component + Serialize + 'static,
	{
		self.handlers.push(TSaveContext::buffer::<TComponent>);

		self
	}

	pub fn save_system_via(context: Arc<Mutex<TSaveContext>>) -> impl Fn(&mut World) {
		move |world| {
			let Ok(mut context) = context.lock() else {
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
		let save: Save = Save::component::<_A>();

		assert_eq!(
			Save {
				handlers: vec![SaveContext::buffer::<_A>]
			},
			save
		);
	}

	#[test]
	fn store_save_fns() {
		let save: Save = Save::component::<_A>().and_component::<_B>();

		assert_eq!(
			Save {
				handlers: vec![SaveContext::buffer::<_A>, SaveContext::buffer::<_B>]
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
		buffered: Vec<(Entity, TypeId)>,
	}

	impl ExecuteSave for _SaveContext {
		fn buffer<T>(&mut self, entity: EntityRef)
		where
			T: 'static,
		{
			self.buffered.push((entity.id(), TypeId::of::<T>()));
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
	fn buffer() {
		let context = Arc::new(Mutex::new(_SaveContext::default()));
		let mut app = setup(context.clone());
		let entity = app
			.world_mut()
			.spawn(Save::<_SaveContext>::component::<_A>().and_component::<_B>())
			.id();

		app.update();

		assert_eq!(
			vec![(entity, TypeId::of::<_A>()), (entity, TypeId::of::<_B>())],
			context.lock().expect("COULD NOT LOCK CONTEXT").buffered
		);
	}
}
