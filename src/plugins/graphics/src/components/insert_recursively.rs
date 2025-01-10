use bevy::prelude::*;
use common::traits::{thread_safe::ThreadSafe, try_insert_on::TryInsertOn};

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct InsertRecursively<T>(pub(crate) T)
where
	T: Component + Clone + ThreadSafe;

impl<T> InsertRecursively<T>
where
	T: Component + Clone + ThreadSafe,
{
	pub(crate) fn apply(
		mut commands: Commands,
		entities: Query<(Entity, &Self)>,
		children: Query<&Children>,
	) {
		for (entity, InsertRecursively(component)) in &entities {
			commands.try_insert_on(entity, component.clone());

			for child in children.iter_descendants(entity) {
				commands.try_insert_on(child, component.clone());
			}
		}
	}
}

impl<T> From<T> for InsertRecursively<T>
where
	T: Component + Clone + ThreadSafe,
{
	fn from(inner: T) -> Self {
		Self(inner)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Component;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, InsertRecursively::<_Component>::apply);

		app
	}

	#[test]
	fn insert_component() {
		let mut app = setup();
		let entity = app.world_mut().spawn(InsertRecursively(_Component)).id();

		app.update();

		assert_eq!(
			Some(&_Component),
			app.world().entity(entity).get::<_Component>()
		)
	}

	#[test]
	fn insert_component_on_children() {
		let mut app = setup();
		let entity = app.world_mut().spawn(InsertRecursively(_Component)).id();
		let entity = app.world_mut().spawn_empty().set_parent(entity).id();

		app.update();

		assert_eq!(
			Some(&_Component),
			app.world().entity(entity).get::<_Component>()
		)
	}

	#[test]
	fn insert_component_on_2nd_generation_children() {
		let mut app = setup();
		let entity = app.world_mut().spawn(InsertRecursively(_Component)).id();
		let entity = app.world_mut().spawn_empty().set_parent(entity).id();
		let entity = app.world_mut().spawn_empty().set_parent(entity).id();

		app.update();

		assert_eq!(
			Some(&_Component),
			app.world().entity(entity).get::<_Component>()
		)
	}
}
