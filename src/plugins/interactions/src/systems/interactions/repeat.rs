use crate::{components::Repeat, traits::ActOn};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		system::{Commands, Query, Res},
	},
	time::Time,
};

pub(crate) fn repeat<
	TActor: ActOn<TTarget> + Clone + Component,
	TTarget: Send + Sync + 'static,
	TTime: Default + Send + Sync + 'static,
>(
	mut commands: Commands,
	time: Res<Time<TTime>>,
	mut repeaters: Query<(Entity, &mut Repeat<TActor, TTarget>)>,
) {
	let delta = time.delta();

	for (id, mut repeater) in &mut repeaters {
		if delta < repeater.timer {
			repeater.timer -= delta;
		} else {
			commands.entity(id).insert(repeater.actor.clone());
			repeater.timer = repeater.after;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::RepeatAfter;
	use bevy::{
		app::{App, Update},
		time::Real,
	};
	use common::test_tools::utils::{SingleThreadedApp, TickTime};
	use std::time::Duration;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Actor;

	struct _Target;

	impl ActOn<_Target> for _Actor {
		fn act_on(&mut self, _: &mut _Target) {}
	}

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, repeat::<_Actor, _Target, Real>);
		app.init_resource::<Time<Real>>();

		app
	}

	#[test]
	fn update_repeater_timer() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(_Actor.repeat_after(Duration::from_millis(42)))
			.id();

		app.tick_time(Duration::from_millis(10));
		app.update();

		let repeater = app
			.world
			.entity(agent)
			.get::<Repeat<_Actor, _Target>>()
			.unwrap();

		assert_eq!(Duration::from_millis(32), repeater.timer);
	}

	#[test]
	fn insert_actor_after_pause() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(_Actor.repeat_after(Duration::from_millis(42)))
			.id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&_Actor), agent.get::<_Actor>());
	}

	#[test]
	fn no_insert_when_pause_time_not_passed() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(_Actor.repeat_after(Duration::from_millis(42)))
			.id();

		app.tick_time(Duration::from_millis(41));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<_Actor>());
	}

	#[test]
	fn insert_actor_after_pause_exceeded() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(_Actor.repeat_after(Duration::from_millis(42)))
			.id();

		app.tick_time(Duration::from_millis(43));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&_Actor), agent.get::<_Actor>());
	}

	#[test]
	fn reset_repeater_timer() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(_Actor.repeat_after(Duration::from_millis(42)))
			.id();

		app.tick_time(Duration::from_millis(21));
		app.update();

		app.tick_time(Duration::from_millis(21));
		app.update();

		let repeater = app
			.world
			.entity(agent)
			.get::<Repeat<_Actor, _Target>>()
			.unwrap();

		assert_eq!(Duration::from_millis(42), repeater.timer);
	}
}
