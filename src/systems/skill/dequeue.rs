use crate::{
	components::{Animate, Queue, Track, WaitNext},
	traits::get_animation::HasIdle,
};
use bevy::prelude::{Commands, Entity, Query, With};

type Components<'a, TAnimationTemplate, TAnimationKey> = (
	Entity,
	&'a mut Queue<TAnimationTemplate>,
	Option<&'a Animate<TAnimationKey>>,
);

pub fn dequeue<
	TAnimationTemplate: Clone + Copy + Sync + Send + 'static,
	TAnimationKey: PartialEq + Clone + Copy + Sync + Send + 'static,
>(
	mut commands: Commands,
	mut agents: Query<Components<TAnimationTemplate, TAnimationKey>, With<WaitNext>>,
) where
	Queue<TAnimationTemplate>: HasIdle<TAnimationKey>,
{
	let idle = &Queue::<TAnimationTemplate>::IDLE;

	for (agent, mut queue, animate) in agents.iter_mut() {
		let mut agent = commands.entity(agent);

		if let Some(skill) = queue.0.pop_front() {
			agent.insert(Track::new(skill.to_active()));
			agent.remove::<WaitNext>();
			if matches!(animate, Some(animate) if animate == idle) {
				agent.remove::<Animate<TAnimationKey>>();
			}
		} else {
			agent.insert(*idle);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Animate, SlotKey, WaitNext},
		skill::{Active, Cast, Queued, SelectInfo, Skill},
	};
	use bevy::prelude::{default, App, Ray, Update, Vec3};
	use std::time::Duration;

	const TEST_RAY: Ray = Ray {
		origin: Vec3::ONE,
		direction: Vec3::Y,
	};

	#[derive(Clone, Copy, Default)]
	struct _Template;

	#[derive(Clone, Copy, PartialEq, Debug)]
	enum _Key {
		Idle,
		NotIdle,
	}

	impl HasIdle<_Key> for Queue<_Template> {
		const IDLE: Animate<_Key> = Animate::Replay(_Key::Idle);
	}

	impl Default for Skill<_Template, Queued> {
		fn default() -> Self {
			Self {
				name: Default::default(),
				data: Default::default(),
				cast: Default::default(),
				soft_override: Default::default(),
				animate: Default::default(),
				behavior: Default::default(),
				is_usable_with: Default::default(),
			}
		}
	}

	#[test]
	fn pop_first_behavior_to_agent() {
		let mut app = App::new();
		let queue = Queue::<_Template>(
			[(Skill {
				cast: Cast {
					pre: Duration::from_millis(42),
					..default()
				},
				data: Queued {
					select_info: SelectInfo {
						ray: TEST_RAY,
						..default()
					},
					slot_key: SlotKey::SkillSpawn,
				},
				..default()
			})]
			.into(),
		);
		let agent = app.world.spawn((queue, WaitNext)).id();

		app.add_systems(Update, dequeue::<_Template, _Key>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<_Template>>().unwrap();

		assert_eq!(
			(
				Some(Active {
					select_info: SelectInfo {
						ray: TEST_RAY,
						..default()
					},
					slot_key: SlotKey::SkillSpawn,
				}),
				false,
				0
			),
			(
				agent
					.get::<Track<Skill<_Template, Active>>>()
					.map(|t| t.value.data.clone()),
				agent.contains::<WaitNext>(),
				queue.0.len()
			)
		);
	}

	#[test]
	fn do_not_pop_when_not_waiting_next() {
		let mut app = App::new();
		let queue = Queue::<_Template>([Skill::default()].into());
		let agent = app.world.spawn(queue).id();

		app.add_systems(Update, dequeue::<_Template, _Key>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<_Template>>().unwrap();

		assert_eq!(
			(false, 1),
			(
				agent.contains::<Track<Skill<_Key, Queued>>>(),
				queue.0.len()
			)
		);
	}

	#[test]
	fn idle_when_nothing_to_pop() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((Queue::<_Template>([].into()), WaitNext))
			.id();

		app.add_systems(Update, dequeue::<_Template, _Key>);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Queue::<_Template>::IDLE),
			agent.get::<Animate<_Key>>()
		);
	}

	#[test]
	fn remove_idle_when_something_to_pop() {
		let mut app = App::new();
		let queue = Queue::<_Template>([Skill::default()].into());

		let agent = app
			.world
			.spawn((queue, WaitNext, Queue::<_Template>::IDLE))
			.id();
		app.add_systems(Update, dequeue::<_Template, _Key>);
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Animate<_Key>>());
	}

	#[test]
	fn do_not_remove_non_idle_when_something_to_pop() {
		let mut app = App::new();
		let queue = Queue::<_Template>([Skill::default()].into());

		let agent = app
			.world
			.spawn((queue, WaitNext, Animate::Replay(_Key::NotIdle)))
			.id();
		app.add_systems(Update, dequeue::<_Template, _Key>);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Animate::Replay(_Key::NotIdle)),
			agent.get::<Animate<_Key>>()
		);
	}
}
