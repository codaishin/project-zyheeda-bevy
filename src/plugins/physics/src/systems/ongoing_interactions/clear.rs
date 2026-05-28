use crate::resources::root_collisions::RootCollisions;
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;

impl<T> RootCollisions<T>
where
	T: ThreadSafe,
{
	pub(crate) fn clear(mut interactions: ResMut<Self>) {
		interactions.rotate();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashSet;
	use testing::{SingleThreadedApp, fake_entity};

	#[derive(Debug, PartialEq)]
	struct _T;

	fn setup(interactions: RootCollisions<_T>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(interactions);
		app.add_systems(Update, RootCollisions::<_T>::clear);

		app
	}

	#[test]
	fn rotate_on_clear() {
		let mut app = setup(RootCollisions::from([(
			fake_entity!(42),
			HashSet::from([]),
		)]));

		app.update();

		let mut rotated = RootCollisions::from([(fake_entity!(42), HashSet::from([]))]);
		rotated.rotate();
		assert_eq!(&rotated, app.world().resource::<RootCollisions<_T>>(),);
	}
}
