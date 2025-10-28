use crate::components::agent::tag::AgentTag;
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{GetProperty, TryApplyOn},
		handles_agents::AgentType,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl AgentTag {
	pub(crate) fn insert_from<TSource>(
		trigger: Trigger<OnInsert, TSource>,
		mut commands: ZyheedaCommands,
		sources: Query<&TSource>,
	) where
		TSource: Component + GetProperty<AgentType>,
	{
		let entity = trigger.target();
		let Ok(source) = sources.get(entity) else {
			return;
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(Self(source.get_property()));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_enemies::EnemyType;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent(AgentType);

	impl GetProperty<AgentType> for _Agent {
		fn get_property(&self) -> AgentType {
			self.0
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(AgentTag::insert_from::<_Agent>);

		app
	}

	#[test_case(AgentType::Player; "player")]
	#[test_case(AgentType::Enemy(EnemyType::VoidSphere); "void sphere")]
	fn insert_tag(agent_type: AgentType) {
		let mut app = setup();

		let entity = app.world_mut().spawn(_Agent(agent_type));

		assert_eq!(Some(&AgentTag(agent_type)), entity.get::<AgentTag>(),)
	}

	#[test_case(AgentType::Player; "player")]
	#[test_case(AgentType::Enemy(EnemyType::VoidSphere); "void sphere")]
	fn re_insert_tag(agent_type: AgentType) {
		let mut app = setup();

		let mut entity = app.world_mut().spawn(_Agent(agent_type));
		entity.remove::<AgentTag>();
		entity.insert(_Agent(agent_type));

		assert_eq!(Some(&AgentTag(agent_type)), entity.get::<AgentTag>(),)
	}
}
