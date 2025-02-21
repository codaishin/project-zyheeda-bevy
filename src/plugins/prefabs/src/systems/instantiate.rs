use crate::components::SpawnAfterInstantiation;
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	traits::prefab::Prefab,
};

pub fn instantiate<TAgent, TDependency>(
	mut commands: Commands,
	agents: Query<(Entity, &TAgent), Added<TAgent>>,
) -> Vec<Result<(), Error>>
where
	TAgent: Component + Prefab<TDependency>,
{
	let instantiate = |(entity, agent): (Entity, &TAgent)| {
		let Some(mut entity) = commands.get_entity(entity) else {
			return Err(Error {
				msg: format!("Cannot instantiate prefab, because {entity:?} does not exist",),
				lvl: Level::Error,
			});
		};
		agent.instantiate_on::<SpawnAfterInstantiation>(&mut entity)
	};

	agents.iter().map(instantiate).collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		errors::Level,
		systems::log::test_tools::{fake_log_error_lazy_many, FakeErrorLogMany},
		traits::prefab::AfterInstantiation,
	};
	use std::marker::PhantomData;
	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Debug, PartialEq)]
	struct _Child;

	#[derive(Component)]
	struct _Result<TAsset: Asset>(Handle<TAsset>);

	impl Prefab<()> for _Agent {
		fn instantiate_on<TAfterInstantiation>(
			&self,
			entity: &mut EntityCommands,
		) -> Result<(), Error>
		where
			TAfterInstantiation: AfterInstantiation,
		{
			entity.try_insert((TAfterInstantiation::spawn(|parent| {
				parent.spawn(_Child);
			}),));
			Ok(())
		}
	}

	#[derive(Component)]
	struct _AgentWithInstantiationError;

	impl Prefab<()> for _AgentWithInstantiationError {
		fn instantiate_on<TAfterInstantiation>(&self, _: &mut EntityCommands) -> Result<(), Error> {
			Err(Error {
				msg: "AAA".to_owned(),
				lvl: Level::Warning,
			})
		}
	}

	#[derive(Resource)]
	struct _Assets<TAsset>(PhantomData<TAsset>);

	impl<T> Default for _Assets<T> {
		fn default() -> Self {
			Self(PhantomData)
		}
	}

	#[derive(Resource)]
	struct _Storage<TAsset>(PhantomData<TAsset>);

	impl<T> Default for _Storage<T> {
		fn default() -> Self {
			Self(PhantomData)
		}
	}

	fn setup<TAgent>() -> (App, Entity)
	where
		TAgent: Component + Prefab<()>,
	{
		let mut app = App::new();
		let logger = app.world_mut().spawn_empty().id();
		let instantiate_system = instantiate::<TAgent, ()>;
		app.add_systems(
			Update,
			instantiate_system.pipe(fake_log_error_lazy_many(logger)),
		);

		(app, logger)
	}

	fn children(app: &App, entity: Entity) -> impl Iterator<Item = EntityRef> {
		app.world().iter_entities().filter(move |child| {
			child
				.get::<Parent>()
				.map(|parent| parent.get() == entity)
				.unwrap_or(false)
		})
	}

	#[test]
	fn add_spawn_after_instantiation_component() -> Result<(), RunSystemError> {
		let (mut app, ..) = setup::<_Agent>();
		let agent = app.world_mut().spawn(_Agent).id();

		app.update();
		let after_instantiation = app
			.world()
			.entity(agent)
			.get::<SpawnAfterInstantiation>()
			.unwrap()
			.clone();
		// Can't compare `SpawnAfterInstantiation` directly (Arc<dyn Fn(..)>), so we apply the spawn
		// function to see that the configured child is spawned correctly
		app.world_mut()
			.run_system_once(move |mut commands: Commands| {
				let mut entity = commands.entity(agent);
				let spawn_on = after_instantiation.spawn.clone();
				entity.with_children(|parent| spawn_on(parent));
			})?;

		assert_eq!(
			vec![&_Child],
			children(&app, agent)
				.filter_map(|child| child.get::<_Child>())
				.collect::<Vec<_>>()
		);
		Ok(())
	}

	#[test]
	fn log_errors() {
		let (mut app, logger) = setup::<_AgentWithInstantiationError>();
		app.world_mut().spawn(_AgentWithInstantiationError);

		app.update();

		let log = app
			.world()
			.entity(logger)
			.get::<FakeErrorLogMany>()
			.unwrap();

		assert_eq!(
			vec![Error {
				msg: "AAA".to_owned(),
				lvl: Level::Warning,
			}],
			log.0
		);
	}
}
