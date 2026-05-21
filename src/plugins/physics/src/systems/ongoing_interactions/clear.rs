use crate::resources::ongoing_interactions::OngoingInteractions;
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;

impl<T> OngoingInteractions<T>
where
	T: ThreadSafe,
{
	pub(crate) fn clear(mut interactions: ResMut<Self>) {
		interactions.interactions.clear();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashSet;
	use testing::{SingleThreadedApp, fake_entity};

	#[derive(Debug, PartialEq)]
	struct _T;

	fn setup(interactions: OngoingInteractions<_T>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(interactions);
		app.add_systems(Update, OngoingInteractions::<_T>::clear);

		app
	}

	#[test]
	fn clear() {
		let mut app = setup(OngoingInteractions::from([(
			fake_entity!(42),
			HashSet::from([]),
		)]));

		app.update();

		assert_eq!(
			&OngoingInteractions::default(),
			app.world().resource::<OngoingInteractions<_T>>(),
		);
	}
}
