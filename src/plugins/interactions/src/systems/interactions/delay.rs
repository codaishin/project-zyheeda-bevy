use crate::{components::Delay, traits::ActOn};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		system::{Commands, Query, Res},
		world::Mut,
	},
	time::Time,
};
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};

pub(crate) fn delay<
	TActor: ActOn<TTarget> + Clone + Component,
	TTarget: Send + Sync + 'static,
	TTime: Default + Send + Sync + 'static,
>(
	mut commands: Commands,
	time: Res<Time<TTime>>,
	mut delays: Query<(Entity, &mut Delay<TActor, TTarget>)>,
) {
	let delta = time.delta();

	for (id, mut delay) in &mut delays {
		if delta < delay.timer {
			delay.timer -= delta;
		} else {
			trigger(delay, &mut commands, id);
		}
	}
}

fn trigger<TActor: ActOn<TTarget> + Clone + Component, TTarget: Send + Sync + 'static>(
	mut delay: Mut<Delay<TActor, TTarget>>,
	commands: &mut Commands,
	id: Entity,
) {
	commands.try_insert_on(id, delay.actor.clone());
	if delay.repeat {
		delay.timer = delay.after;
	} else {
		commands.try_remove_from::<Delay<TActor, TTarget>>(id);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{InitDelay, Repeat};
	use bevy::{
		app::{App, Update},
		time::Real,
	};
	use common::test_tools::utils::{SingleThreadedApp, TickTime};
	use std::time::Duration;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Actor;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Target;

	impl ActOn<_Target> for _Actor {
		fn act_on(&mut self, _: &mut _Target) {}
	}

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, delay::<_Actor, _Target, Real>);
		app.init_resource::<Time<Real>>();

		app
	}

	#[test]
	fn update_repeater_timer() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(_Actor.after(Duration::from_millis(42)))
			.id();

		app.tick_time(Duration::from_millis(10));
		app.update();

		let repeater = app
			.world
			.entity(agent)
			.get::<Delay<_Actor, _Target>>()
			.unwrap();

		assert_eq!(Duration::from_millis(32), repeater.timer);
	}

	#[test]
	fn insert_actor_after_pause() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(_Actor.after(Duration::from_millis(42)))
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
			.spawn(_Actor.after(Duration::from_millis(42)))
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
			.spawn(_Actor.after(Duration::from_millis(42)))
			.id();

		app.tick_time(Duration::from_millis(43));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&_Actor), agent.get::<_Actor>());
	}

	#[test]
	fn remove_delay_when_repeat_not_set() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(_Actor.after(Duration::from_millis(42)))
			.id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let repeater = app.world.entity(agent).get::<Delay<_Actor, _Target>>();

		assert_eq!(None, repeater);
	}

	#[test]
	fn reset_repeater_timer() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(_Actor.after(Duration::from_millis(42)).repeat())
			.id();

		app.tick_time(Duration::from_millis(21));
		app.update();

		app.tick_time(Duration::from_millis(21));
		app.update();

		let repeater = app
			.world
			.entity(agent)
			.get::<Delay<_Actor, _Target>>()
			.unwrap();

		assert_eq!(Duration::from_millis(42), repeater.timer);
	}
}
