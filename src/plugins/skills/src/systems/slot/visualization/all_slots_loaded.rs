use crate::components::slots::visualization::SlotVisualization;
use bevy::prelude::*;
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::{AssociatedSystemParam, GetFromSystemParam},
		handles_load_tracking::Loaded,
		thread_safe::ThreadSafe,
		visible_slots::VisibleSlots,
	},
};
use std::hash::Hash;

impl<TKey> SlotVisualization<TKey>
where
	TKey: Eq + Hash + From<SlotKey> + ThreadSafe,
{
	pub(crate) fn all_slots_loaded_for<TAgent>(
		agents: Query<(&TAgent, &Self)>,
		param: AssociatedSystemParam<TAgent, ()>,
	) -> Loaded
	where
		TAgent: Component + GetFromSystemParam<()>,
		for<'i> TAgent::TItem<'i>: VisibleSlots,
	{
		let all_visible_slots_discovered = agents
			.iter()
			.filter_map(|(agent, visualization)| {
				agent
					.get_from_param(&(), &param)
					.map(|slots| (slots, visualization))
			})
			.all(|(slots, visualization)| {
				slots
					.visible_slots()
					.map(TKey::from)
					.all(|key| visualization.slots.contains_key(&key))
			});

		Loaded(all_visible_slots_discovered)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{tools::action_key::slot::SlotKey, traits::visible_slots::VisibleSlots};
	use testing::SingleThreadedApp;

	#[derive(PartialEq, Eq, Hash)]
	struct _Key(SlotKey);

	impl From<SlotKey> for _Key {
		fn from(slot_key: SlotKey) -> Self {
			Self(slot_key)
		}
	}

	#[derive(Component)]
	struct _Agent;

	impl GetFromSystemParam<()> for _Agent {
		type TParam<'w, 's> = ();
		type TItem<'i> = _VisibleSlots;

		fn get_from_param(&self, _: &(), _: &()) -> Option<_VisibleSlots> {
			Some(_VisibleSlots)
		}
	}

	struct _VisibleSlots;

	impl VisibleSlots for _VisibleSlots {
		fn visible_slots(&self) -> impl Iterator<Item = SlotKey> {
			[SlotKey(1), SlotKey(2)].into_iter()
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn all_loaded() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			SlotVisualization::from([
				(_Key(SlotKey(1)), Entity::from_raw(42)),
				(_Key(SlotKey(2)), Entity::from_raw(42)),
			]),
		));

		let loaded = app
			.world_mut()
			.run_system_once(SlotVisualization::<_Key>::all_slots_loaded_for::<_Agent>)?;

		assert_eq!(Loaded(true), loaded);
		Ok(())
	}

	#[test]
	fn not_all_loaded() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			SlotVisualization::from([
				(_Key(SlotKey(1)), Entity::from_raw(42)),
				(_Key(SlotKey(255)), Entity::from_raw(42)),
			]),
		));

		let loaded = app
			.world_mut()
			.run_system_once(SlotVisualization::<_Key>::all_slots_loaded_for::<_Agent>)?;

		assert_eq!(Loaded(false), loaded);
		Ok(())
	}
}
