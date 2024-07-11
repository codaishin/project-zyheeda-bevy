use crate::traits::Cleanup;
use bevy::ecs::{
	component::Component,
	entity::Entity,
	system::{Commands, In, Query},
};
use common::components::Idle;
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub(crate) struct SetToIdle<T> {
	pub entities: Vec<Entity>,
	phantom_data: PhantomData<T>,
}

impl<T> SetToIdle<T> {
	pub fn new(entities: Vec<Entity>) -> Self {
		Self {
			entities,
			phantom_data: PhantomData,
		}
	}
}

pub(crate) fn idle<TCleanup: Component + Cleanup>(
	set_to_idle: In<SetToIdle<TCleanup>>,
	mut commands: Commands,
	cleanups: Query<&TCleanup>,
) {
	for entity in set_to_idle.entities.iter() {
		let Ok(cleanup) = cleanups.get(*entity) else {
			continue;
		};
		let Some(entity) = &mut commands.get_entity(*entity) else {
			continue;
		};
		cleanup.cleanup(entity);
		entity.try_insert(Idle);
		entity.remove::<TCleanup>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::{EntityCommands, IntoSystem, Res, Resource},
	};
	use common::{components::Idle, test_tools::utils::SingleThreadedApp};

	#[derive(Component, Debug, PartialEq)]
	struct _Cleaned;

	#[derive(Component, Debug, PartialEq)]
	struct _Cleanup;

	impl Cleanup for _Cleanup {
		fn cleanup(&self, agent: &mut EntityCommands) {
			agent.insert(_Cleaned);
		}
	}

	#[derive(Resource, Default)]
	struct _DoIdle(Vec<Entity>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_DoIdle>();
		app.add_systems(
			Update,
			(|c: Res<_DoIdle>| SetToIdle::new(c.0.clone())).pipe(idle::<_Cleanup>),
		);

		app
	}

	#[test]
	fn add_idle() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Cleanup).id();
		app.insert_resource(_DoIdle(vec![agent]));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&Idle), agent.get::<Idle>());
	}

	#[test]
	fn do_not_add_idle_when_no_cleanup_present() {
		let mut app = setup();
		let agent = app.world_mut().spawn_empty().id();
		app.insert_resource(_DoIdle(vec![agent]));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<Idle>());
	}

	#[test]
	fn call_cleanup() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Cleanup).id();
		app.insert_resource(_DoIdle(vec![agent]));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Cleaned), agent.get::<_Cleaned>());
	}

	#[test]
	fn remove_cleanup_component() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Cleanup).id();
		app.insert_resource(_DoIdle(vec![agent]));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_Cleanup>());
	}
}
