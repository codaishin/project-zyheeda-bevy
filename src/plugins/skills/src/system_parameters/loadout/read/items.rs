use crate::{
	components::{inventory::Inventory, slots::Slots},
	item::Item,
	system_parameters::loadout::LoadoutReader,
};
use bevy::prelude::*;
use common::{
	tools::inventory_key::InventoryKey,
	traits::{
		accessors::get::{ContextChanged, GetContext, GetProperty},
		handles_loadout::{
			LoadoutKey,
			items::{ItemToken, Items, ReadItems},
		},
		handles_localization::Token,
	},
};

impl GetContext<Items> for LoadoutReader<'_, '_> {
	type TContext<'ctx> = ItemsView<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx LoadoutReader,
		Items { entity }: Items,
	) -> Option<Self::TContext<'ctx>> {
		let (slots, inventory, _, _) = param.agents.get(entity).ok()?;

		Some(ItemsView {
			inventory,
			slots,
			items: &param.items,
		})
	}
}

pub struct ItemsView<'a> {
	inventory: Ref<'a, Inventory>,
	slots: Ref<'a, Slots>,
	items: &'a Assets<Item>,
}

impl ContextChanged for ItemsView<'_> {
	fn context_changed(&self) -> bool {
		self.slots.is_changed() || self.inventory.is_changed()
	}
}

impl ReadItems for ItemsView<'_> {
	type TItem<'a>
		= ReadItem
	where
		Self: 'a;

	fn get_item<TKey>(&self, key: TKey) -> Option<Self::TItem<'_>>
	where
		TKey: Into<LoadoutKey>,
	{
		let handle = match key.into() {
			LoadoutKey::Inventory(InventoryKey(i)) => self.inventory.0.get(i)?.as_ref()?,
			LoadoutKey::Slot(slot) => self.slots.items.get(&slot)?.as_ref()?,
		};
		let item = self.items.get(handle)?;

		Some(ReadItem {
			token: item.token.clone(),
		})
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReadItem {
	token: Token,
}

impl GetProperty<ItemToken> for ReadItem {
	fn get_property(&self) -> &'_ Token {
		&self.token
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{components::combos::Combos, skills::Skill};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::{action_key::slot::SlotKey, inventory_key::InventoryKey};
	use testing::{SingleThreadedApp, new_handle};

	mod get_item {
		use super::*;
		use crate::components::queue::Queue;

		fn setup<const N: usize>(items: [(&Handle<Item>, Item); N]) -> App {
			let mut app = App::new().single_threaded(Update);
			let mut item_assets = Assets::default();

			for (id, asset) in items {
				_ = item_assets.insert(id, asset);
			}

			app.insert_resource(item_assets);
			app.init_resource::<Assets<Skill>>();

			app
		}

		#[test]
		fn slot_item() -> Result<(), RunSystemError> {
			let handle = new_handle();
			let item = Item {
				token: Token::from("my item"),
				..default()
			};
			let mut app = setup([(&handle, item)]);
			let entity = app
				.world_mut()
				.spawn((
					Slots::from([(SlotKey(11), Some(handle))]),
					Inventory::default(),
					Combos::default(),
					Queue::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |loadout: LoadoutReader| {
					let ctx = LoadoutReader::get_context(&loadout, Items { entity }).unwrap();
					let item = ctx.get_item(SlotKey(11));

					assert_eq!(
						Some(ReadItem {
							token: Token::from("my item")
						}),
						item
					);
				})
		}

		#[test]
		fn inventory_item() -> Result<(), RunSystemError> {
			let handle = new_handle();
			let item = Item {
				token: Token::from("my item"),
				..default()
			};
			let mut app = setup([(&handle, item)]);
			let entity = app
				.world_mut()
				.spawn((
					Slots::default(),
					Inventory::from([None, None, None, Some(handle), None]),
					Combos::default(),
					Queue::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |loadout: LoadoutReader| {
					let ctx = LoadoutReader::get_context(&loadout, Items { entity }).unwrap();
					let item = ctx.get_item(InventoryKey(3));

					assert_eq!(
						Some(ReadItem {
							token: Token::from("my item")
						}),
						item
					);
				})
		}
	}

	mod item {
		use super::*;

		#[test]
		fn get_token() {
			let item = ReadItem {
				token: Token::from("my item"),
			};

			assert_eq!(&Token::from("my item"), item.get_property());
		}
	}
}
