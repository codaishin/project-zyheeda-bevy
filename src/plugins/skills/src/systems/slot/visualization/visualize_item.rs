use crate::{
	components::slots::{Slots, visualization::SlotVisualization},
	item::Item,
	traits::visualize_item::VisualizeItem,
};
use bevy::prelude::*;
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::{GetProperty, GetRef, TryApplyOn},
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::hash::Hash;

impl<TKey> SlotVisualization<TKey>
where
	TKey: Eq + Hash + ThreadSafe + VisualizeItem + GetProperty<SlotKey>,
{
	pub(crate) fn visualize_items(
		mut commands: ZyheedaCommands,
		slots: Query<(&Slots, &Self), AnyChanged<Slots, Self>>,
		items: Res<Assets<Item>>,
	) {
		for (slots, visualization) in &slots {
			for (key, entity) in &visualization.slots {
				let item = slots
					.get_ref(&key.get_property())
					.and_then(|slot| items.get(slot));

				commands.try_apply_on(entity, |mut e| {
					e.try_insert(TKey::visualize(item));
				});
			}
		}
	}
}

type AnyChanged<T0, T1> = Or<(Changed<T0>, Changed<T1>)>;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::slots::Slots, item::Item};
	use bevy::app::{App, Update};
	use common::{tools::action_key::slot::SlotKey, traits::handles_localization::Token};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Debug, PartialEq, Eq, Hash)]
	struct _Key(SlotKey);

	impl GetProperty<SlotKey> for _Key {
		fn get_property(&self) -> SlotKey {
			self.0
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Component(Option<Item>);

	impl VisualizeItem for _Key {
		type TComponent = _Component;

		fn visualize(item: Option<&Item>) -> Self::TComponent {
			_Component(item.cloned())
		}
	}

	fn setup<const N: usize>(items: [(&Handle<Item>, Item); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut item_assets = Assets::default();

		for (handle, item) in items {
			item_assets.insert(handle, item);
		}

		app.insert_resource(item_assets);
		app.add_systems(Update, SlotVisualization::<_Key>::visualize_items);

		app
	}

	#[test]
	fn visualize() {
		let handle = new_handle();
		let mut app = setup([(
			&handle,
			Item {
				token: Token::from("my-item"),
				..default()
			},
		)]);
		let slot = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			SlotVisualization::from([(_Key(SlotKey(32)), slot)]),
			Slots::from([(SlotKey(32), Some(handle))]),
		));

		app.update();

		assert_eq!(
			Some(&_Component(Some(Item {
				token: Token::from("my-item"),
				..default()
			}))),
			app.world().entity(slot).get::<_Component>()
		);
	}

	#[test]
	fn visualize_missing_asset() {
		let handle = new_handle();
		let mut app = setup([]);
		let slot = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			SlotVisualization::from([(_Key(SlotKey(32)), slot)]),
			Slots::from([(SlotKey(32), Some(handle))]),
		));

		app.update();

		assert_eq!(
			Some(&_Component(None)),
			app.world().entity(slot).get::<_Component>()
		);
	}

	#[test]
	fn visualize_empty_slot() {
		let handle = new_handle();
		let mut app = setup([(
			&handle,
			Item {
				token: Token::from("my-item"),
				..default()
			},
		)]);
		let slot = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			SlotVisualization::from([(_Key(SlotKey(32)), slot)]),
			Slots::from([(SlotKey(32), None)]),
		));

		app.update();

		assert_eq!(
			Some(&_Component(None)),
			app.world().entity(slot).get::<_Component>()
		);
	}

	#[test]
	fn act_only_once() {
		let handle = new_handle();
		let mut app = setup([(
			&handle,
			Item {
				token: Token::from("my-item"),
				..default()
			},
		)]);
		let slot = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			SlotVisualization::from([(_Key(SlotKey(32)), slot)]),
			Slots::from([(SlotKey(32), Some(handle))]),
		));

		app.update();
		app.world_mut().entity_mut(slot).remove::<_Component>();
		app.update();

		assert_eq!(None, app.world().entity(slot).get::<_Component>());
	}

	#[test]
	fn act_again_when_slots_changed() {
		let handle = new_handle();
		let mut app = setup([(
			&handle,
			Item {
				token: Token::from("my-item"),
				..default()
			},
		)]);
		let slot = app.world_mut().spawn_empty().id();
		let slots = app
			.world_mut()
			.spawn((
				SlotVisualization::from([(_Key(SlotKey(32)), slot)]),
				Slots::from([(SlotKey(32), Some(handle))]),
			))
			.id();

		app.update();
		app.world_mut().entity_mut(slot).remove::<_Component>();
		app.world_mut()
			.entity_mut(slots)
			.get_mut::<Slots>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&_Component(Some(Item {
				token: Token::from("my-item"),
				..default()
			}))),
			app.world().entity(slot).get::<_Component>()
		);
	}

	#[test]
	fn act_again_when_slot_visualization_changed() {
		let handle = new_handle();
		let mut app = setup([(
			&handle,
			Item {
				token: Token::from("my-item"),
				..default()
			},
		)]);
		let slot = app.world_mut().spawn_empty().id();
		let slots = app
			.world_mut()
			.spawn((
				SlotVisualization::from([(_Key(SlotKey(32)), slot)]),
				Slots::from([(SlotKey(32), Some(handle))]),
			))
			.id();

		app.update();
		app.world_mut().entity_mut(slot).remove::<_Component>();
		app.world_mut()
			.entity_mut(slots)
			.get_mut::<SlotVisualization<_Key>>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&_Component(Some(Item {
				token: Token::from("my-item"),
				..default()
			}))),
			app.world().entity(slot).get::<_Component>()
		);
	}
}
