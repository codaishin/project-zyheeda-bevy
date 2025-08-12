use crate::{
	components::lifetime::Lifetime,
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};
use bevy::prelude::*;
use std::time::Duration;

impl Lifetime {
	pub(crate) fn update<TTime: Default + Sync + Send + 'static>(
		mut commands: ZyheedaCommands,
		mut lifetimes: Query<(Entity, &mut Lifetime)>,
		time: Res<Time<TTime>>,
	) {
		let delta = time.delta();

		for (entity, mut lifetime) in &mut lifetimes {
			if delta < lifetime.0 {
				lifetime.0 -= delta;
			} else {
				commands.try_apply_on(&entity, |e| e.try_despawn());
			}
		}
	}
}

impl From<Duration> for Lifetime {
	fn from(duration: Duration) -> Self {
		Self(duration)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::time::Duration;
	use testing::{SingleThreadedApp, TickTime};

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
	fn despawn_when_lifetime_zero() {
		let mut app = setup();
		let lifetime = app
			.world_mut()
			.spawn(Lifetime(Duration::from_secs(100)))
			.id();

		app.tick_time(Duration::from_secs(100));
		app.update();

		assert!(app.world().get_entity(lifetime).is_err());
	}

	#[test]
	fn despawn_when_lifetime_below_zero() {
		let mut app = setup();
		let lifetime = app
			.world_mut()
			.spawn(Lifetime(Duration::from_secs(100)))
			.id();

		app.tick_time(Duration::from_secs(101));
		app.update();

		assert!(app.world().get_entity(lifetime).is_err());
	}
}
