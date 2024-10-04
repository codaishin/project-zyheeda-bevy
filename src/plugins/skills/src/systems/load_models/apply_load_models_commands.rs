use crate::{
	components::{slots::Slots, LoadModel, LoadModelsCommand},
	items::{Item, Mount},
};
use bevy::{
	asset::Handle,
	prelude::{default, Commands, Entity, Query, Res},
	scene::Scene,
};
use common::{
	errors::{Error, Level},
	resources::Models,
	traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};

pub(crate) fn apply_load_models_commands(
	mut commands: Commands,
	models: Res<Models>,
	agents: Query<(Entity, &Slots, &LoadModelsCommand)>,
) -> Vec<Result<(), Error>> {
	agents
		.iter()
		.flat_map(|(agent, slots, load_models_command)| {
			load_models_command
				.0
				.iter()
				.filter_map(collect_model_data(agent, slots, &models))
		})
		.map(|model_data| model_data.insert(&mut commands))
		.collect()
}

fn collect_model_data<'a>(
	agent: Entity,
	slots: &'a Slots,
	models: &'a Models,
) -> impl FnMut(&LoadModel) -> Option<LoadData> + 'a {
	move |&LoadModel(slot_key)| {
		let slot = slots.0.get(&slot_key)?;
		let item = slot.item.as_ref()?;

		Some(LoadData {
			agent,
			hand: slot.mounts.hand,
			forearm: slot.mounts.forearm,
			models: try_get_handles(item, models),
		})
	}
}

fn try_get_handles(item: &Item, models: &Models) -> Result<Handles, Error> {
	let Some(model) = item.model else {
		return Ok(Handles {
			hand: default(),
			forearm: default(),
		});
	};
	let Some(model) = models.0.get(model) else {
		return Err(model_error(item));
	};

	match item.mount {
		Mount::Hand => Ok(Handles {
			hand: model.clone(),
			forearm: default(),
		}),
		Mount::Forearm => Ok(Handles {
			hand: default(),
			forearm: model.clone(),
		}),
	}
}

fn model_error(item: &Item) -> Error {
	Error {
		msg: format!(
			"Item({}): no model '{:?}' seems to exist, abandoning",
			item.name, item.model
		),
		lvl: Level::Error,
	}
}

struct LoadData {
	agent: Entity,
	hand: Entity,
	forearm: Entity,
	models: Result<Handles, Error>,
}

struct Handles {
	hand: Handle<Scene>,
	forearm: Handle<Scene>,
}

impl LoadData {
	fn insert(self, commands: &mut Commands) -> Result<(), Error> {
		let (hand_model, forearm_model, result) = match self.models {
			Ok(Handles { hand, forearm }) => (hand, forearm, Ok(())),
			Err(error) => (default(), default(), Err(error)),
		};

		commands.try_remove_from::<LoadModelsCommand>(self.agent);
		commands.try_insert_on(self.hand, hand_model);
		commands.try_insert_on(self.forearm, forearm_model);

		result
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{LoadModel, Mounts, Slot},
		items::{slot_key::SlotKey, Item, Mount},
		skills::Skill,
	};
	use bevy::{
		app::{App, Update},
		asset::AssetId,
		prelude::IntoSystem,
		scene::Scene,
		utils::default,
	};
	use common::{
		components::Side,
		resources::Models,
		systems::log::test_tools::{fake_log_error_many_recourse, FakeErrorLogManyResource},
		test_tools::utils::SingleThreadedApp,
	};
	use std::collections::HashMap;
	use uuid::Uuid;

	fn setup(models: Models) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(models);
		app.add_systems(
			Update,
			apply_load_models_commands.pipe(fake_log_error_many_recourse),
		);

		app
	}

	fn new_handle() -> Handle<Scene> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	#[test]
	fn load_hand_model() {
		let hand_model = new_handle();
		let mut app = setup(Models(HashMap::from([(
			"my/model/path",
			hand_model.clone(),
		)])));

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			Slots::<Skill>::new([(
				SlotKey::Hand(Side::Off),
				Slot {
					mounts: Mounts { hand, forearm },
					item: Some(Item {
						model: Some("my/model/path"),
						mount: Mount::Hand,
						..default()
					}),
				},
			)]),
			LoadModelsCommand::new([LoadModel(SlotKey::Hand(Side::Off))]),
		));

		app.update();

		let hand = app.world().entity(hand);
		let forearm = app.world().entity(forearm);

		assert_eq!(
			(Some(&hand_model), Some(&Handle::default())),
			(hand.get::<Handle<Scene>>(), forearm.get::<Handle<Scene>>())
		);
	}

	#[test]
	fn load_forearm_model() {
		let hand_model = new_handle();
		let mut app = setup(Models(HashMap::from([(
			"my/model/path",
			hand_model.clone(),
		)])));

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			Slots::<Skill>::new([(
				SlotKey::Hand(Side::Off),
				Slot {
					mounts: Mounts { hand, forearm },
					item: Some(Item {
						model: Some("my/model/path"),
						mount: Mount::Forearm,
						..default()
					}),
				},
			)]),
			LoadModelsCommand::new([LoadModel(SlotKey::Hand(Side::Off))]),
		));

		app.update();

		let hand = app.world().entity(hand);
		let forearm = app.world().entity(forearm);

		assert_eq!(
			(Some(&Handle::default()), Some(&hand_model)),
			(hand.get::<Handle<Scene>>(), forearm.get::<Handle<Scene>>())
		);
	}

	#[test]
	fn default_models_when_model_path_none() {
		let mut app = setup(Models(HashMap::from([("my/model/path", new_handle())])));

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			Slots::<Skill>::new([(
				SlotKey::Hand(Side::Off),
				Slot {
					mounts: Mounts { hand, forearm },
					item: Some(Item {
						model: None,
						..default()
					}),
				},
			)]),
			LoadModelsCommand::new([LoadModel(SlotKey::Hand(Side::Off))]),
		));

		app.update();

		let hand = app.world().entity(hand);
		let forearm = app.world().entity(forearm);

		assert_eq!(
			(Some(&Handle::default()), Some(&Handle::default())),
			(hand.get::<Handle<Scene>>(), forearm.get::<Handle<Scene>>())
		);
	}

	#[test]
	fn default_models_when_model_faulty() {
		let mut app = setup(Models(HashMap::from([("my/model/path", new_handle())])));

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			Slots::<Skill>::new([(
				SlotKey::Hand(Side::Off),
				Slot {
					mounts: Mounts { hand, forearm },
					item: Some(Item {
						model: Some("my/faulty/model/path"),
						..default()
					}),
				},
			)]),
			LoadModelsCommand::new([LoadModel(SlotKey::Hand(Side::Off))]),
		));

		app.update();

		let hand = app.world().entity(hand);
		let forearm = app.world().entity(forearm);

		assert_eq!(
			(Some(&Handle::default()), Some(&Handle::default())),
			(hand.get::<Handle<Scene>>(), forearm.get::<Handle<Scene>>())
		);
	}

	#[test]
	fn log_error_when_model_path_faulty() {
		let mut app = setup(Models(HashMap::from([("my/model/path", new_handle())])));

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			Slots::<Skill>::new([(
				SlotKey::Hand(Side::Off),
				Slot {
					mounts: Mounts { hand, forearm },
					item: Some(Item {
						name: "my faulty item",
						model: Some("my/faulty/model/path"),
						..default()
					}),
				},
			)]),
			LoadModelsCommand::new([LoadModel(SlotKey::Hand(Side::Off))]),
		));

		app.update();

		let errors = app.world().get_resource::<FakeErrorLogManyResource>();

		assert_eq!(
			Some(&FakeErrorLogManyResource(vec![model_error(&Item {
				name: "my faulty item",
				model: Some("my/faulty/model/path"),
				..default()
			})])),
			errors,
		)
	}

	#[test]
	fn remove_load_command() {
		let hand_model = new_handle();
		let mut app = setup(Models(HashMap::from([(
			"my/model/path",
			hand_model.clone(),
		)])));

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		let agent = app
			.world_mut()
			.spawn((
				Slots::<Skill>::new([(
					SlotKey::Hand(Side::Off),
					Slot {
						mounts: Mounts { hand, forearm },
						item: Some(Item {
							model: Some("my/model/path"),
							mount: Mount::Hand,
							..default()
						}),
					},
				)]),
				LoadModelsCommand::new([LoadModel(SlotKey::Hand(Side::Off))]),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<LoadModelsCommand>());
	}
}
