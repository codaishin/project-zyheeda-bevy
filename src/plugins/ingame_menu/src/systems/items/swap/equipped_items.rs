use bevy::{
	asset::Handle,
	ecs::{
		query::QueryEntityError,
		system::{Commands, Query},
	},
	prelude::{Entity, Mut},
	scene::Scene,
};
use common::{
	components::{Collection, Swap},
	errors::{Error, Level},
	traits::try_remove_from::TryRemoveFrom,
};
use skills::{
	components::{slots::Slots, Slot},
	items::slot_key::SlotKey,
	skills::Skill,
};

type SlotsToSwap<'a> = (
	Entity,
	&'a mut Slots<Handle<Skill>>,
	&'a Collection<Swap<SlotKey, SlotKey>>,
);

pub fn swap_equipped_items(
	mut commands: Commands,
	mut slots_to_swap: Query<SlotsToSwap>,
	mut handles: Query<&mut Handle<Scene>>,
) -> Vec<Result<(), Error>> {
	let mut results = vec![];

	for (agent, mut slots, swaps) in &mut slots_to_swap {
		for swap in &swaps.0 {
			results.extend_from_slice(&do_swap(swap, &mut slots, &mut handles));
		}

		commands.try_remove_from::<Collection<Swap<SlotKey, SlotKey>>>(agent);
	}

	results
}

fn do_swap(
	swap: &Swap<SlotKey, SlotKey>,
	slots: &mut Mut<Slots<Handle<Skill>>>,
	handles: &mut Query<&mut Handle<Scene>>,
) -> [Result<(), Error>; 2] {
	let slot_results = [
		slots.0.get(&swap.0).cloned().ok_or(no_slot(swap.0)),
		slots.0.get(&swap.1).cloned().ok_or(no_slot(swap.1)),
	];

	let [Ok(slot0), Ok(slot1)] = slot_results else {
		return slot_results.map(drop_ok);
	};

	let handle_results = [
		get_handles(&slot0, handles).map_err(no_handle(swap.0)),
		get_handles(&slot1, handles).map_err(no_handle(swap.1)),
	];

	let [Ok((h0_hand, h0_forearm)), Ok((h1_hand, h1_forearm))] = handle_results else {
		return handle_results.map(drop_ok);
	};

	if let Some(slot) = slots.0.get_mut(&swap.0) {
		slot.item = slot1.item;
	}
	if let Some(slot) = slots.0.get_mut(&swap.1) {
		slot.item = slot0.item;
	}
	if let Ok(mut handle) = handles.get_mut(slot0.mounts.hand) {
		*handle = h1_hand;
	}
	if let Ok(mut handle) = handles.get_mut(slot0.mounts.forearm) {
		*handle = h1_forearm;
	}
	if let Ok(mut handle) = handles.get_mut(slot1.mounts.hand) {
		*handle = h0_hand;
	}
	if let Ok(mut handle) = handles.get_mut(slot1.mounts.forearm) {
		*handle = h0_forearm;
	}

	[Ok(()), Ok(())]
}

fn get_handles(
	slot: &Slot<Handle<Skill>>,
	handles: &mut Query<&mut Handle<Scene>>,
) -> Result<(Handle<Scene>, Handle<Scene>), QueryEntityError> {
	Ok((
		handles.get(slot.mounts.hand).cloned()?,
		handles.get(slot.mounts.forearm).cloned()?,
	))
}

fn drop_ok<V>(result: Result<V, Error>) -> Result<(), Error> {
	match result {
		Err(error) => Err(error),
		Ok(_) => Ok(()),
	}
}

fn no_slot(slot_key: SlotKey) -> Error {
	Error {
		msg: format!("{:?}: Slot not found", slot_key),
		lvl: Level::Error,
	}
}

fn no_handle(slot_key: SlotKey) -> impl Fn(QueryEntityError) -> Error {
	move |_| handle_error(slot_key)
}

fn handle_error(slot_key: SlotKey) -> Error {
	Error {
		msg: format!("{:?}: Handle not found", slot_key),
		lvl: Level::Error,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::AssetId,
		ecs::system::In,
		prelude::{default, Entity, IntoSystem},
		utils::Uuid,
	};
	use common::{
		components::Side,
		systems::log::test_tools::{fake_log_error_lazy_many, FakeErrorLogMany},
	};
	use skills::{
		components::{Mounts, Slot},
		items::{Item, Mount},
		skills::Skill,
	};

	#[test]
	fn swap_items() {
		let mut app = App::new();
		let slot_handles = [
			Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}),
			Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}),
			Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}),
			Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}),
		];
		let slot_handle_ids = slot_handles
			.clone()
			.map(|handle| app.world.spawn(handle.clone()).id());
		let agent = app
			.world
			.spawn((
				Slots(
					[
						(
							SlotKey::Hand(Side::Off),
							Slot::<Handle<Skill>> {
								mounts: Mounts {
									hand: slot_handle_ids[0],
									forearm: slot_handle_ids[1],
								},
								item: Some(Item {
									name: "left item",
									mount: Mount::Forearm,
									..default()
								}),
							},
						),
						(
							SlotKey::Hand(Side::Main),
							Slot {
								mounts: Mounts {
									hand: slot_handle_ids[2],
									forearm: slot_handle_ids[3],
								},
								item: Some(Item {
									name: "right item",
									mount: Mount::Hand,
									..default()
								}),
							},
						),
					]
					.into(),
				),
				Collection([Swap(SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main))].into()),
			))
			.id();

		app.add_systems(Update, swap_equipped_items.pipe(|_: In<_>| {}));
		app.update();

		let handles =
			slot_handle_ids.map(|id| app.world.entity(id).get::<Handle<Scene>>().unwrap());
		let slots = app
			.world
			.entity(agent)
			.get::<Slots<Handle<Skill>>>()
			.unwrap();
		let new_items = (
			slots.0.get(&SlotKey::Hand(Side::Off)).unwrap().item.clone(),
			slots
				.0
				.get(&SlotKey::Hand(Side::Main))
				.unwrap()
				.item
				.clone(),
		);
		let errors = app.world.entity(agent).get::<FakeErrorLogMany>();

		assert_eq!(
			(
				[
					&slot_handles[2],
					&slot_handles[3],
					&slot_handles[0],
					&slot_handles[1],
				],
				(
					Some(Item {
						name: "right item",
						mount: Mount::Hand,
						..default()
					}),
					Some(Item {
						name: "left item",
						mount: Mount::Forearm,
						..default()
					})
				),
				None
			),
			(handles, new_items, errors)
		);
	}

	#[test]
	fn remove_collection() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((
				Slots::<Handle<Skill>>([].into()),
				Collection::<Swap<SlotKey, SlotKey>>([].into()),
			))
			.id();

		app.add_systems(Update, swap_equipped_items.pipe(|_: In<_>| {}));
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Collection<Swap<SlotKey, SlotKey>>>());
	}

	#[test]
	fn log_slot_errors() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((
				Slots::<Handle<Skill>>([].into()),
				Collection([Swap(SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main))].into()),
			))
			.id();

		app.add_systems(
			Update,
			swap_equipped_items.pipe(fake_log_error_lazy_many(agent)),
		);
		app.update();

		let errors = app.world.entity(agent).get::<FakeErrorLogMany>().unwrap();

		assert_eq!(
			vec![
				no_slot(SlotKey::Hand(Side::Off)),
				no_slot(SlotKey::Hand(Side::Main))
			],
			errors.0
		)
	}

	#[test]
	fn log_handle_errors() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((
				Slots(
					[
						(
							SlotKey::Hand(Side::Off),
							Slot::<Handle<Skill>> {
								mounts: Mounts {
									hand: Entity::from_raw(100),
									forearm: Entity::from_raw(200),
								},
								item: Some(Item {
									name: "left item",
									..default()
								}),
							},
						),
						(
							SlotKey::Hand(Side::Main),
							Slot {
								mounts: Mounts {
									hand: Entity::from_raw(101),
									forearm: Entity::from_raw(202),
								},
								item: Some(Item {
									name: "right item",
									..default()
								}),
							},
						),
					]
					.into(),
				),
				Collection([Swap(SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main))].into()),
			))
			.id();

		app.add_systems(
			Update,
			swap_equipped_items.pipe(fake_log_error_lazy_many(agent)),
		);
		app.update();

		let errors = app.world.entity(agent).get::<FakeErrorLogMany>().unwrap();

		assert_eq!(
			vec![
				handle_error(SlotKey::Hand(Side::Off)),
				handle_error(SlotKey::Hand(Side::Main))
			],
			errors.0
		)
	}
}
