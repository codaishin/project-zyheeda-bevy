use crate::components::LifeTime;
use bevy::prelude::*;
use common::{components::destroy::Destroy, traits::try_insert_on::TryInsertOn};

pub(crate) fn update_lifetimes<TTime: Default + Sync + Send + 'static>(
	mut commands: Commands,
	mut lifetimes: Query<(Entity, &mut LifeTime, Option<&Destroy>)>,
	time: Res<Time<TTime>>,
) {
	let delta = time.delta();

	for (id, mut lifetime, despawn) in &mut lifetimes {
		if delta < lifetime.0 {
			lifetime.0 -= delta;
		} else if despawn.is_none() {
			commands.try_insert_on(id, Destroy::DELAYED);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::{SingleThreadedApp, TickTime};
	use std::time::Duration;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, update_lifetimes::<Real>);
		app.init_resource::<Time<Real>>();

		app
	}

	#[test]
	fn decrease_lifetime_by_delta() {
		let mut app = setup();
		let lifetime = app
			.world_mut()
			.spawn(LifeTime(Duration::from_secs(100)))
			.id();

		app.tick_time(Duration::from_secs(10));
		app.update();

		let lifetime = app.world().entity(lifetime).get::<LifeTime>();

		assert_eq!(Some(&LifeTime(Duration::from_secs(90))), lifetime);
	}

	#[test]
	fn mark_despawn_when_lifetime_zero() {
		let mut app = setup();
		let lifetime = app
			.world_mut()
			.spawn(LifeTime(Duration::from_secs(100)))
			.id();

		app.tick_time(Duration::from_secs(100));
		app.update();

		let lifetime = app.world().entity(lifetime);

		assert_eq!(Some(&Destroy::DELAYED), lifetime.get::<Destroy>());
	}

	#[test]
	fn mark_despawn_when_lifetime_below_zero() {
		let mut app = setup();
		let lifetime = app
			.world_mut()
			.spawn(LifeTime(Duration::from_secs(100)))
			.id();

		app.tick_time(Duration::from_secs(101));
		app.update();

		let lifetime = app.world().entity(lifetime);

		assert_eq!(Some(&Destroy::DELAYED), lifetime.get::<Destroy>());
	}

	#[test]
	fn do_not_add_despawn_when_already_present() {
		let mut app = setup();
		let lifetime = app
			.world_mut()
			.spawn((
				LifeTime(Duration::from_secs(100)),
				Destroy::AfterFrames(100),
			))
			.id();

		app.tick_time(Duration::from_secs(100));
		app.update();

		let lifetime = app.world().entity(lifetime);

		assert_eq!(Some(&Destroy::AfterFrames(100)), lifetime.get::<Destroy>());
	}
}
