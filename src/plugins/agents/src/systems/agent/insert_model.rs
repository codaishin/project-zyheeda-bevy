use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{AssociatedSystemParam, GetFromSystemParam, TryApplyOn},
		handles_agents::AgentConfig,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

impl<T> InsertModelSystem for T where
	T: Component + for<'a> GetFromSystemParam<AgentConfig, TItem<'a>: InsertModel>
{
}

pub(crate) trait InsertModelSystem:
	Component + for<'a> GetFromSystemParam<AgentConfig, TItem<'a>: InsertModel> + Sized
{
	fn insert_model(
		mut commands: ZyheedaCommands,
		param: AssociatedSystemParam<Self, AgentConfig>,
		agents: Query<(Entity, &Self), Without<ModelInserted>>,
	) {
		for (entity, agent) in &agents {
			let Some(config) = agent.get_from_param(&AgentConfig, &param) else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				config.insert_model(&mut e);
				e.try_insert(ModelInserted);
			});
		}
	}
}

#[derive(Component)]
pub(crate) struct ModelInserted;

pub(crate) trait InsertModel {
	fn insert_model(&self, entity: &mut ZyheedaEntityCommands);
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	impl GetFromSystemParam<AgentConfig> for _Agent {
		type TParam<'world, 'state> = ();
		type TItem<'item> = _Data;

		fn get_from_param(&self, _: &AgentConfig, _: &()) -> Option<_Data> {
			Some(_Data)
		}
	}

	#[derive(Clone)]
	struct _Data;

	impl InsertModel for _Data {
		fn insert_model(&self, entity: &mut ZyheedaEntityCommands) {
			entity.try_insert(_Model);
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Model;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, _Agent::insert_model);

		app
	}

	#[test]
	fn insert_model() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Agent).id();

		app.update();

		assert_eq!(Some(&_Model), app.world().entity(entity).get::<_Model>());
	}

	#[test]
	fn insert_model_only_once() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Agent).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<_Model>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Model>());
	}
}
