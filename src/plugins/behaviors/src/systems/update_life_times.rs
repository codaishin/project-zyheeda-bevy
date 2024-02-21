use crate::components::LifeTime;
use bevy::{
	ecs::{
		entity::Entity,
		system::{Commands, Query, Res},
	},
	hierarchy::DespawnRecursiveExt,
	time::Time,
};

pub(crate) fn update_lifetimes<TTime: Default + Sync + Send + 'static>(
	mut commands: Commands,
	mut lifetimes: Query<(Entity, &mut LifeTime)>,
	time: Res<Time<TTime>>,
) {
	let delta = time.delta();

	for (id, mut lifetime) in &mut lifetimes {
		if delta < lifetime.0 {
			lifetime.0 -= delta;
		} else {
			commands.entity(id).despawn_recursive();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		hierarchy::BuildWorldChildren,
		time::Real,
	};
	use common::test_tools::utils::{SingleThreadedApp, TickTime};
	use std::time::Duration;

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, update_lifetimes::<Real>);
		app.init_resource::<Time<Real>>();

		app
	}

	#[test]
	fn decrease_lifetime_by_delta() {
		let mut app = setup();
		let lifetime = app.world.spawn(LifeTime(Duration::from_secs(100))).id();

		app.tick_time(Duration::from_secs(10));
		app.update();

		let lifetime = app.world.entity(lifetime).get::<LifeTime>();

		assert_eq!(Some(&LifeTime(Duration::from_secs(90))), lifetime);
	}

	#[test]
	fn despawn_when_lifetime_zero() {
		let mut app = setup();
		let lifetime = app.world.spawn(LifeTime(Duration::from_secs(100))).id();

		app.tick_time(Duration::from_secs(100));
		app.update();

		let lifetime = app.world.get_entity(lifetime);

		assert!(lifetime.is_none());
	}

	#[test]
	fn despawn_when_lifetime_below_zero() {
		let mut app = setup();
		let lifetime = app.world.spawn(LifeTime(Duration::from_secs(100))).id();

		app.tick_time(Duration::from_secs(101));
		app.update();

		let lifetime = app.world.get_entity(lifetime);

		assert!(lifetime.is_none());
	}

	#[test]
	fn despawn_recursive() {
		let mut app = setup();
		let lifetime = app.world.spawn(LifeTime(Duration::from_secs(100))).id();
		let child = app.world.spawn_empty().set_parent(lifetime).id();

		app.tick_time(Duration::from_secs(100));
		app.update();

		let child = app.world.get_entity(child);

		assert!(child.is_none());
	}
}
