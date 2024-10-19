use crate::{
	components::{slots::Slots, LoadModel, LoadModelsCommand},
	items::{slot_key::SlotKey, Item, Mount},
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	resources::Models,
	traits::{get::GetRef, try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};

pub(crate) fn apply_load_models_commands<THands, TForearms>(
	mut commands: Commands,
	models: Res<Models>,
	agents: Query<(Entity, &Slots, &LoadModelsCommand, &THands, &TForearms)>,
) -> Vec<Result<(), Error>>
where
	THands: Component + GetRef<SlotKey, Entity>,
	TForearms: Component + GetRef<SlotKey, Entity>,
{
	agents
		.iter()
		.flat_map(|(entity, slots, cmd, hands, forearms)| {
			cmd.0
				.iter()
				.map(collect_model_data(entity, slots, &models, hands, forearms))
		})
		.filter_map(|model_data| match model_data {
			Err(ModelDataError::SlotEmpty) => None,
			Err(ModelDataError::NoMountEntity(key, mount)) => {
				Some(Err(slot_entity_error(&key, &mount)))
			}
			Ok(model_data) => match model_data.insert(&mut commands) {
				Ok(()) => None,
				Err(error) => Some(Err(error)),
			},
		})
		.collect()
}

enum ModelDataError {
	SlotEmpty,
	NoMountEntity(SlotKey, Mount),
}

fn collect_model_data<'a, THandMounts, TForearmMounts>(
	agent: Entity,
	slots: &'a Slots,
	models: &'a Models,
	hand_mounts: &'a THandMounts,
	forearm_mounts: &'a TForearmMounts,
) -> impl FnMut(&LoadModel) -> Result<LoadData, ModelDataError> + 'a
where
	THandMounts: Component + GetRef<SlotKey, Entity>,
	TForearmMounts: Component + GetRef<SlotKey, Entity>,
{
	move |&LoadModel(slot_key)| {
		let item = slots
			.0
			.get(&slot_key)
			.and_then(|item| item.as_ref())
			.ok_or(ModelDataError::SlotEmpty)?;
		let hand_mount = hand_mounts
			.get(&slot_key)
			.ok_or(ModelDataError::NoMountEntity(slot_key, Mount::Hand))?;
		let forearm_mount = forearm_mounts
			.get(&slot_key)
			.ok_or(ModelDataError::NoMountEntity(slot_key, Mount::Forearm))?;

		Ok(LoadData {
			agent,
			hand: *hand_mount,
			forearm: *forearm_mount,
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

fn slot_entity_error(key: &SlotKey, mount: &Mount) -> Error {
	Error {
		msg: format!("No {:?} slot entity for {:?} found, abandoning", mount, key,),
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
		components::LoadModel,
		items::{slot_key::SlotKey, Item, Mount},
		skills::Skill,
	};
	use bevy::ecs::system::RunSystemOnce;
	use common::{components::Side, resources::Models, test_tools::utils::SingleThreadedApp};
	use std::collections::HashMap;
	use uuid::Uuid;

	#[derive(Component, Default)]
	struct _Hands(HashMap<SlotKey, Entity>);

	impl GetRef<SlotKey, Entity> for _Hands {
		fn get(&self, key: &SlotKey) -> Option<&Entity> {
			self.0.get(key)
		}
	}

	#[derive(Component, Default)]
	struct _Forearms(HashMap<SlotKey, Entity>);

	impl GetRef<SlotKey, Entity> for _Forearms {
		fn get(&self, key: &SlotKey) -> Option<&Entity> {
			self.0.get(key)
		}
	}

	fn setup(models: Models) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(models);

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
			_Hands([(SlotKey::BottomHand(Side::Left), hand)].into()),
			_Forearms([(SlotKey::BottomHand(Side::Left), forearm)].into()),
			Slots::<Skill>::new([(
				SlotKey::BottomHand(Side::Left),
				Some(Item {
					model: Some("my/model/path"),
					mount: Mount::Hand,
					..default()
				}),
			)]),
			LoadModelsCommand::new([LoadModel(SlotKey::BottomHand(Side::Left))]),
		));

		app.world_mut()
			.run_system_once(apply_load_models_commands::<_Hands, _Forearms>);

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
			_Hands([(SlotKey::BottomHand(Side::Left), hand)].into()),
			_Forearms([(SlotKey::BottomHand(Side::Left), forearm)].into()),
			Slots::<Skill>::new([(
				SlotKey::BottomHand(Side::Left),
				Some(Item {
					model: Some("my/model/path"),
					mount: Mount::Forearm,
					..default()
				}),
			)]),
			LoadModelsCommand::new([LoadModel(SlotKey::BottomHand(Side::Left))]),
		));

		app.world_mut()
			.run_system_once(apply_load_models_commands::<_Hands, _Forearms>);

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
			_Hands([(SlotKey::BottomHand(Side::Left), hand)].into()),
			_Forearms([(SlotKey::BottomHand(Side::Left), forearm)].into()),
			Slots::<Skill>::new([(
				SlotKey::BottomHand(Side::Left),
				Some(Item {
					model: None,
					..default()
				}),
			)]),
			LoadModelsCommand::new([LoadModel(SlotKey::BottomHand(Side::Left))]),
		));

		app.world_mut()
			.run_system_once(apply_load_models_commands::<_Hands, _Forearms>);

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
			_Hands([(SlotKey::BottomHand(Side::Left), hand)].into()),
			_Forearms([(SlotKey::BottomHand(Side::Left), forearm)].into()),
			Slots::<Skill>::new([(
				SlotKey::BottomHand(Side::Left),
				Some(Item {
					model: Some("my/faulty/model/path"),
					..default()
				}),
			)]),
			LoadModelsCommand::new([LoadModel(SlotKey::BottomHand(Side::Left))]),
		));

		app.world_mut()
			.run_system_once(apply_load_models_commands::<_Hands, _Forearms>);

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
			_Hands([(SlotKey::BottomHand(Side::Left), hand)].into()),
			_Forearms([(SlotKey::BottomHand(Side::Left), forearm)].into()),
			Slots::<Skill>::new([(
				SlotKey::BottomHand(Side::Left),
				Some(Item {
					name: "my faulty item",
					model: Some("my/faulty/model/path"),
					..default()
				}),
			)]),
			LoadModelsCommand::new([LoadModel(SlotKey::BottomHand(Side::Left))]),
		));

		let errors = app
			.world_mut()
			.run_system_once(apply_load_models_commands::<_Hands, _Forearms>);

		assert_eq!(
			vec![Err(model_error(&Item {
				name: "my faulty item",
				model: Some("my/faulty/model/path"),
				..default()
			}))],
			errors,
		)
	}

	#[test]
	fn log_error_when_slot_entity_not_set_for_hand() {
		let mut app = setup(Models(HashMap::from([("my/model/path", new_handle())])));

		let forearm = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			_Hands::default(),
			_Forearms([(SlotKey::BottomHand(Side::Left), forearm)].into()),
			Slots::<Skill>::new([(
				SlotKey::BottomHand(Side::Left),
				Some(Item {
					name: "my faulty item",
					model: Some("my/faulty/model/path"),
					..default()
				}),
			)]),
			LoadModelsCommand::new([LoadModel(SlotKey::BottomHand(Side::Left))]),
		));

		let errors = app
			.world_mut()
			.run_system_once(apply_load_models_commands::<_Hands, _Forearms>);

		assert_eq!(
			vec![Err(slot_entity_error(
				&SlotKey::BottomHand(Side::Left),
				&Mount::Hand
			))],
			errors,
		)
	}

	#[test]
	fn log_error_when_slot_entity_not_set_for_forearm() {
		let mut app = setup(Models(HashMap::from([("my/model/path", new_handle())])));

		let hand = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			_Hands([(SlotKey::BottomHand(Side::Left), hand)].into()),
			_Forearms::default(),
			Slots::<Skill>::new([(
				SlotKey::BottomHand(Side::Left),
				Some(Item {
					name: "my faulty item",
					model: Some("my/faulty/model/path"),
					..default()
				}),
			)]),
			LoadModelsCommand::new([LoadModel(SlotKey::BottomHand(Side::Left))]),
		));

		let errors = app
			.world_mut()
			.run_system_once(apply_load_models_commands::<_Hands, _Forearms>);

		assert_eq!(
			vec![Err(slot_entity_error(
				&SlotKey::BottomHand(Side::Left),
				&Mount::Forearm
			))],
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
				_Hands([(SlotKey::BottomHand(Side::Left), hand)].into()),
				_Forearms([(SlotKey::BottomHand(Side::Left), forearm)].into()),
				Slots::<Skill>::new([(
					SlotKey::BottomHand(Side::Left),
					Some(Item {
						model: Some("my/model/path"),
						mount: Mount::Hand,
						..default()
					}),
				)]),
				LoadModelsCommand::new([LoadModel(SlotKey::BottomHand(Side::Left))]),
			))
			.id();

		app.world_mut()
			.run_system_once(apply_load_models_commands::<_Hands, _Forearms>);

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<LoadModelsCommand>());
	}
}
