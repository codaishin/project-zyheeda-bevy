use crate::components::agent::Agent;
use bevy::{
	ecs::{
		query::{QueryFilter, QueryIter},
		system::SystemParam,
	},
	prelude::*,
};
use common::components::persistent_entity::PersistentEntity;
use std::iter::Copied;

#[derive(SystemParam)]
pub struct AgentParam<'w, 's, TFilter>
where
	TFilter: QueryFilter + 'static,
{
	agents: Query<'w, 's, &'static PersistentEntity, (With<Agent>, TFilter)>,
}

impl<'w, 's, TFilter> IntoIterator for AgentParam<'w, 's, TFilter>
where
	TFilter: QueryFilter + 'static,
{
	type Item = PersistentEntity;
	type IntoIter = Copied<QueryIter<'w, 's, &'static PersistentEntity, (With<Agent>, TFilter)>>;

	fn into_iter(self) -> Self::IntoIter {
		self.agents.into_iter().copied()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::{handles_enemies::EnemyType, handles_map_generation::AgentType};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn return_entities() -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = PersistentEntity::default();
		let b = PersistentEntity::default();
		app.world_mut().spawn((
			a,
			Agent {
				agent_type: AgentType::Player,
			},
		));
		app.world_mut().spawn((
			b,
			Agent {
				agent_type: AgentType::Enemy(EnemyType::VoidSphere),
			},
		));

		let entities = app
			.world_mut()
			.run_system_once(|a: AgentParam<()>| a.into_iter().collect::<Vec<_>>())?;

		assert_eq!(vec![a, b], entities);
		Ok(())
	}

	#[test]
	fn skip_non_agents() -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = PersistentEntity::default();
		let b = PersistentEntity::default();
		let c = PersistentEntity::default();
		app.world_mut().spawn((
			a,
			Agent {
				agent_type: AgentType::Player,
			},
		));
		app.world_mut().spawn(b);
		app.world_mut().spawn((
			c,
			Agent {
				agent_type: AgentType::Enemy(EnemyType::VoidSphere),
			},
		));

		let entities = app
			.world_mut()
			.run_system_once(|a: AgentParam<()>| a.into_iter().collect::<Vec<_>>())?;

		assert_eq!(vec![a, c], entities);
		Ok(())
	}

	#[test]
	fn skip_mismatching_filter() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct Skip;

		let mut app = setup();
		let a = PersistentEntity::default();
		let b = PersistentEntity::default();
		let c = PersistentEntity::default();
		app.world_mut().spawn((
			a,
			Agent {
				agent_type: AgentType::Player,
			},
		));
		app.world_mut().spawn((
			b,
			Skip,
			Agent {
				agent_type: AgentType::Enemy(EnemyType::VoidSphere),
			},
		));
		app.world_mut().spawn((
			c,
			Agent {
				agent_type: AgentType::Enemy(EnemyType::VoidSphere),
			},
		));

		let entities = app
			.world_mut()
			.run_system_once(|a: AgentParam<Without<Skip>>| a.into_iter().collect::<Vec<_>>())?;

		assert_eq!(vec![a, c], entities);
		Ok(())
	}
}
