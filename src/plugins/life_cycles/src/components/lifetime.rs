use super::destroy::Destroy;
use bevy::prelude::*;
use common::{impl_savable_self_non_priority, traits::try_insert_on::TryInsertOn};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct Lifetime(pub(crate) Duration);

impl Lifetime {
	pub(crate) fn update<TTime: Default + Sync + Send + 'static>(
		mut commands: Commands,
		mut lifetimes: Query<(Entity, &mut Lifetime)>,
		time: Res<Time<TTime>>,
	) {
		let delta = time.delta();

		for (id, mut lifetime) in &mut lifetimes {
			if delta < lifetime.0 {
				lifetime.0 -= delta;
			} else {
				commands.try_insert_on(id, Destroy);
			}
		}
	}
}

impl_savable_self_non_priority!(Lifetime);

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::{SingleThreadedApp, TickTime};
	use std::time::Duration;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, Lifetime::update::<Real>);
		app.init_resource::<Time<Real>>();

		app
	}

	#[test]
	fn decrease_lifetime_by_delta() {
		let mut app = setup();
		let lifetime = app
			.world_mut()
			.spawn(Lifetime(Duration::from_secs(100)))
			.id();

		app.tick_time(Duration::from_secs(10));
		app.update();

		let lifetime = app.world().entity(lifetime).get::<Lifetime>();

		assert_eq!(Some(&Lifetime(Duration::from_secs(90))), lifetime);
	}

	#[test]
	fn mark_despawn_when_lifetime_zero() {
		let mut app = setup();
		let lifetime = app
			.world_mut()
			.spawn(Lifetime(Duration::from_secs(100)))
			.id();

		app.tick_time(Duration::from_secs(100));
		app.update();

		let lifetime = app.world().entity(lifetime);

		assert_eq!(Some(&Destroy), lifetime.get::<Destroy>());
	}

	#[test]
	fn mark_despawn_when_lifetime_below_zero() {
		let mut app = setup();
		let lifetime = app
			.world_mut()
			.spawn(Lifetime(Duration::from_secs(100)))
			.id();

		app.tick_time(Duration::from_secs(101));
		app.update();

		let lifetime = app.world().entity(lifetime);

		assert_eq!(Some(&Destroy), lifetime.get::<Destroy>());
	}
}
