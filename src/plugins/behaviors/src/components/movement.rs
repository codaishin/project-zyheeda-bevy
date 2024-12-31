pub(crate) mod velocity_based;

use crate::traits::{Cleanup, RemoveComponent};
use bevy::prelude::*;
use common::test_tools::utils::ApproxEqual;
use std::marker::PhantomData;

#[derive(Component, Clone, PartialEq, Debug, Default)]
pub struct Movement<TMovement> {
	pub(crate) target: Vec3,
	pub(crate) cleanup: Option<fn(&mut EntityCommands)>,
	phantom_data: PhantomData<TMovement>,
}

impl<TMovement> Movement<TMovement> {
	pub fn to(target: Vec3) -> Self {
		Self {
			target,
			cleanup: None,
			phantom_data: PhantomData,
		}
	}

	pub fn remove_on_cleanup<TBundle: Bundle>(self) -> Self {
		Self {
			target: self.target,
			cleanup: Some(TBundle::get_remover()),
			phantom_data: self.phantom_data,
		}
	}
}

impl<TMovement> ApproxEqual<f32> for Movement<TMovement> {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		self.target.approx_equal(&other.target, tolerance) && self.cleanup == other.cleanup
	}
}

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
