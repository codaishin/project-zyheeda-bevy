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
	components::{Collection, SlotKey, Slots, Swap},
	errors::{Error, Level},
};

type SlotsToSwap<'a> = (
	Entity,
	&'a mut Slots,
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

		let mut agent = commands.entity(agent);
		agent.remove::<Collection<Swap<SlotKey, SlotKey>>>();
	}

	results
}

fn do_swap(
	swap: &Swap<SlotKey, SlotKey>,
	slots: &mut Mut<Slots>,
	handles: &mut Query<&mut Handle<Scene>>,
) -> [Result<(), Error>; 2] {
	let slot_results = [
		slots.0.get(&swap.0).cloned().ok_or(no_slot(swap.0)),
		slots.0.get(&swap.1).cloned().ok_or(no_slot(swap.1)),
	];

	let [Ok(s0), Ok(s1)] = slot_results else {
		return slot_results.map(drop_ok);
	};

	let handle_results = [
		handles.get(s0.entity).cloned().map_err(no_handle(swap.0)),
		handles.get(s1.entity).cloned().map_err(no_handle(swap.1)),
	];

	let [Ok(h0), Ok(h1)] = handle_results else {
		return handle_results.map(drop_ok);
	};

	_ = slots.0.get_mut(&swap.0).map(|s| s.item = s1.item);
	_ = slots.0.get_mut(&swap.1).map(|s| s.item = s0.item);
	_ = handles.get_mut(s0.entity).map(|mut h| *h = h1);
	_ = handles.get_mut(s1.entity).map(|mut s| *s = h0);

	[Ok(()), Ok(())]
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
		components::{Item, Side, Slot},
		systems::log::test_tools::{fake_log_error_lazy_many, FakeErrorLogMany},
	};

	#[test]
	fn swap_items() {
		let mut app = App::new();
		let slot_handle_left = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let slot_handle_right = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let slot_handle_left_id = app.world.spawn(slot_handle_left.clone()).id();
		let slot_handle_right_id = app.world.spawn(slot_handle_right.clone()).id();
		let agent = app
			.world
			.spawn((
				Slots(
					[
						(
							SlotKey::Hand(Side::Off),
							Slot {
								entity: slot_handle_left_id,
								item: Some(Item {
									name: "left item",
									..default()
								}),
								combo_skill: None,
							},
						),
						(
							SlotKey::Hand(Side::Main),
							Slot {
								entity: slot_handle_right_id,
								item: Some(Item {
									name: "right item",
									..default()
								}),
								combo_skill: None,
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

		let new_handles = (
			app.world
				.entity(slot_handle_left_id)
				.get::<Handle<Scene>>()
				.unwrap(),
			app.world
				.entity(slot_handle_right_id)
				.get::<Handle<Scene>>()
				.unwrap(),
		);
		let slots = app.world.entity(agent).get::<Slots>().unwrap();
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
				(&slot_handle_right, &slot_handle_left),
				(
					Some(Item {
						name: "right item",
						..default()
					}),
					Some(Item {
						name: "left item",
						..default()
					})
				),
				None
			),
			(new_handles, new_items, errors)
		);
	}

	#[test]
	fn remove_collection() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((
				Slots([].into()),
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
				Slots([].into()),
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
							Slot {
								entity: Entity::from_raw(42),
								item: Some(Item {
									name: "left item",
									..default()
								}),
								combo_skill: None,
							},
						),
						(
							SlotKey::Hand(Side::Main),
							Slot {
								entity: Entity::from_raw(43),
								item: Some(Item {
									name: "right item",
									..default()
								}),
								combo_skill: None,
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
