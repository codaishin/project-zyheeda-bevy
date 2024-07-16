use super::RemoveComponent;
use bevy::ecs::{bundle::Bundle, system::EntityCommands};

impl<T: Bundle> RemoveComponent<T> for T {
	fn get_remover() -> fn(&mut EntityCommands) {
		remove_fn::<T>
	}
}

fn remove_fn<T: Bundle>(entity: &mut EntityCommands) {
	entity.remove::<T>();
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::{
			component::Component,
			entity::Entity,
			system::{Commands, Query},
		},
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _A;

	#[derive(Component, Debug, PartialEq)]
	struct _B;

	fn call_remover<TBundle: Bundle>(mut commands: Commands, query: Query<Entity>) {
		let remove_bundle = TBundle::get_remover();
		for id in &query {
			remove_bundle(&mut commands.entity(id));
		}
	}

	#[test]
	fn removes_bundle() {
		let mut app = App::new().single_threaded(Update);
		let entity = app.world_mut().spawn((_A, _B)).id();

		app.add_systems(Update, call_remover::<(_A, _B)>);
		app.update();

		let entity = app.world().entity(entity);

		assert_eq!((None, None), (entity.get::<_A>(), entity.get::<_B>()));
	}
}
