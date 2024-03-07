use crate::components::{Queue, Track};
use bevy::prelude::{Commands, Entity, Query, With};
use common::components::Idle;

type Components<'a, TAnimationTemplate> = (Entity, &'a mut Queue<TAnimationTemplate>);

pub(crate) fn dequeue<TAnimationTemplate: Clone + Copy + Sync + Send + 'static>(
	mut commands: Commands,
	mut agents: Query<Components<TAnimationTemplate>, With<Idle>>,
) {
	for (agent, mut queue) in agents.iter_mut() {
		let mut agent = commands.entity(agent);

		let Some(skill) = queue.0.pop_front() else {
			continue;
		};
		agent.insert(Track::new(skill.to_active()));
		agent.remove::<Idle>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::SlotKey,
		skill::{Active, Cast, Queued, SelectInfo, Skill},
	};
	use bevy::{
		math::{primitives::Direction3d, Ray3d},
		prelude::{default, App, Update, Vec3},
	};
	use std::time::Duration;

	fn test_ray() -> Ray3d {
		Ray3d {
			origin: Vec3::ONE,
			direction: Direction3d::Y,
		}
	}

	#[derive(Clone, Copy, Default)]
	struct _Template;

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
					target: SelectInfo {
						ray: test_ray(),
						..default()
					},
					slot_key: SlotKey::SkillSpawn,
				},
				..default()
			})]
			.into(),
		);
		let agent = app.world.spawn((queue, Idle)).id();

		app.add_systems(Update, dequeue::<_Template>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<_Template>>().unwrap();

		assert_eq!(
			(
				Some(Active {
					target: SelectInfo {
						ray: test_ray(),
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
				agent.contains::<Idle>(),
				queue.0.len()
			)
		);
	}

	#[test]
	fn do_not_pop_when_not_idle() {
		let mut app = App::new();
		let queue = Queue::<_Template>([Skill::default()].into());
		let agent = app.world.spawn(queue).id();

		app.add_systems(Update, dequeue::<_Template>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<_Template>>().unwrap();

		assert_eq!(
			(false, 1),
			(
				agent.contains::<Track<Skill<_Template, Queued>>>(),
				queue.0.len()
			)
		);
	}
}
