pub(crate) mod position_based;
pub(crate) mod velocity_based;

use super::Cleanup;
use crate::components::Movement;
use bevy::ecs::system::EntityCommands;

impl<T> Cleanup for Movement<T> {
	fn cleanup(&self, agent: &mut EntityCommands) {
		let Some(cleanup) = self.cleanup else {
			return;
		};
		(cleanup)(agent);
	}
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
		prelude::default,
	};
	use common::test_tools::utils::SingleThreadedApp;

	struct _T;

	#[test]
	fn use_cleanup() {
		#[derive(Component)]
		struct _RemoveMe;

		fn call_cleanup(mut commands: Commands, query: Query<(Entity, &Movement<_T>)>) {
			for (id, movement) in &query {
				movement.cleanup(&mut commands.entity(id));
			}
		}

		let mut app = App::new().single_threaded(Update);
		let movement = Movement::<_T>::to(default()).remove_on_cleanup::<_RemoveMe>();
		let movement = app.world_mut().spawn((movement, _RemoveMe)).id();

		app.add_systems(Update, call_cleanup);
		app.update();

		let movement = app.world().entity(movement);

		assert!(!movement.contains::<_RemoveMe>());
	}
}
