use crate::{
	components::{acted_on_targets::ActedOnTargets, Delay},
	traits::ActOn,
};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		system::{Commands, Query},
		world::Mut,
	},
	prelude::In,
};
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};
use std::time::Duration;

pub(crate) fn delay<TActor: ActOn<TTarget> + Clone + Component, TTarget: Send + Sync + 'static>(
	In(delta): In<Duration>,
	mut commands: Commands,
	mut delays: Query<(Entity, &mut Delay<TActor, TTarget>)>,
) {
	for (entity, mut delay) in &mut delays {
		if delta < delay.timer {
			delay.timer -= delta;
		} else {
			trigger(delay, &mut commands, entity);
		}
	}
}

fn trigger<TActor: ActOn<TTarget> + Clone + Component, TTarget: Send + Sync + 'static>(
	mut delay: Mut<Delay<TActor, TTarget>>,
	commands: &mut Commands,
	entity: Entity,
) {
	commands.try_insert_on(
		entity,
		(delay.actor.clone(), ActedOnTargets::<TActor>::default()),
	);
	if delay.repeat {
		delay.timer = delay.after;
	} else {
		commands.try_remove_from::<Delay<TActor, TTarget>>(entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{InitDelay, Repeat},
		traits::ActionType,
	};
	use bevy::{app::App, ecs::system::RunSystemOnce};
	use std::time::Duration;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Actor;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Target;

	impl ActOn<_Target> for _Actor {
		fn act(&mut self, _: Entity, _: &mut _Target, _: Duration) -> ActionType {
			ActionType::Always
		}
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn update_repeater_timer() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(_Actor.after(Duration::from_millis(42)))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_millis(10), delay::<_Actor, _Target>);

		let repeater = app
			.world()
			.entity(agent)
			.get::<Delay<_Actor, _Target>>()
			.unwrap();

		assert_eq!(Duration::from_millis(32), repeater.timer);
	}

	#[test]
	fn insert_actor_after_pause() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(_Actor.after(Duration::from_millis(42)))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_millis(42), delay::<_Actor, _Target>);

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Actor), agent.get::<_Actor>());
	}

	#[test]
	fn insert_empty_acted_on_targets_after_pause() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				ActedOnTargets::<_Actor>::new([Entity::from_raw(100)]),
				_Actor.after(Duration::from_millis(42)),
			))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_millis(42), delay::<_Actor, _Target>);

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&ActedOnTargets::<_Actor>::default()),
			agent.get::<ActedOnTargets<_Actor>>()
		);
	}

	#[test]
	fn no_insert_when_pause_time_not_passed() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(_Actor.after(Duration::from_millis(42)))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_millis(41), delay::<_Actor, _Target>);

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_Actor>());
	}

	#[test]
	fn insert_actor_after_pause_exceeded() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(_Actor.after(Duration::from_millis(42)))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_millis(43), delay::<_Actor, _Target>);

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Actor), agent.get::<_Actor>());
	}

	#[test]
	fn remove_delay_when_repeat_not_set() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(_Actor.after(Duration::from_millis(42)))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_millis(42), delay::<_Actor, _Target>);

		let repeater = app.world().entity(agent).get::<Delay<_Actor, _Target>>();

		assert_eq!(None, repeater);
	}

	#[test]
	fn reset_repeater_timer() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(_Actor.after(Duration::from_millis(42)).repeat())
			.id();

		app.world_mut()
			.run_system_once_with(Duration::from_millis(21), delay::<_Actor, _Target>);

		app.world_mut()
			.run_system_once_with(Duration::from_millis(21), delay::<_Actor, _Target>);

		let repeater = app
			.world()
			.entity(agent)
			.get::<Delay<_Actor, _Target>>()
			.unwrap();

		assert_eq!(Duration::from_millis(42), repeater.timer);
	}
}
