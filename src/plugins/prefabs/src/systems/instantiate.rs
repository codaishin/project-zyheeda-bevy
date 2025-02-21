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
		agent.instantiate_on(&mut entity)
	};

	agents.iter().map(instantiate).collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		errors::Level,
		systems::log::test_tools::{fake_log_error_lazy_many, FakeErrorLogMany},
	};
	use std::marker::PhantomData;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Debug, PartialEq)]
	struct _Component;

	#[derive(Component)]
	struct _Result<TAsset: Asset>(Handle<TAsset>);

	impl Prefab<()> for _Agent {
		fn instantiate_on(&self, entity: &mut EntityCommands) -> Result<(), Error> {
			entity.try_insert(_Component);
			Ok(())
		}
	}

	#[derive(Component)]
	struct _AgentWithInstantiationError;

	impl Prefab<()> for _AgentWithInstantiationError {
		fn instantiate_on(&self, _: &mut EntityCommands) -> Result<(), Error> {
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

	#[test]
	fn insert_component() {
		let (mut app, ..) = setup::<_Agent>();
		let agent = app.world_mut().spawn(_Agent).id();

		app.update();

		assert!(app.world().entity(agent).contains::<_Component>());
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
