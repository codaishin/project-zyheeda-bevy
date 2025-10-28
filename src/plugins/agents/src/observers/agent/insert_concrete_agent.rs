use bevy::prelude::*;
use common::{
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

impl<T> InsertConcreteAgent for T where T: Component + InsertEnemyOrPlayer + Sized {}

pub(crate) trait InsertConcreteAgent: Component + InsertEnemyOrPlayer + Sized {
	fn insert_concrete_agent(
		trigger: Trigger<OnInsert, Self>,
		mut commands: ZyheedaCommands,
		agents: Query<&Self>,
	) {
		let entity = trigger.target();
		let Ok(agent) = agents.get(entity) else {
			return;
		};

		commands.try_apply_on(&entity, |e| {
			agent.insert_enemy_or_player(e);
		});
	}
}

pub(crate) trait InsertEnemyOrPlayer {
	fn insert_enemy_or_player(&self, entity: ZyheedaEntityCommands);
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent(&'static str);

	impl InsertEnemyOrPlayer for _Agent {
		fn insert_enemy_or_player(&self, mut entity: ZyheedaEntityCommands) {
			entity.try_insert(_Concrete(self.0));
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Concrete(&'static str);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(_Agent::insert_concrete_agent);

		app
	}

	#[test]
	fn insert_concrete() {
		let mut app = setup();

		let entity = app.world_mut().spawn(_Agent("my agent"));

		assert_eq!(Some(&_Concrete("my agent")), entity.get::<_Concrete>());
	}

	#[test]
	fn reinsert_concrete_again_when_agent_reinserted() {
		let mut app = setup();

		let mut entity = app.world_mut().spawn(_Agent("1"));
		entity.insert(_Agent("2"));

		assert_eq!(Some(&_Concrete("2")), entity.get::<_Concrete>());
	}
}
