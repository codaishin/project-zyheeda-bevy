mod read;
mod write;

use crate::{
	components::{combos::Combos, inventory::Inventory, queue::Queue, slots::Slots},
	item::Item,
	skills::Skill,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{traits::load_asset::LoadAsset, zyheeda_commands::ZyheedaCommands};

#[derive(SystemParam)]
pub struct LoadoutReader<'w, 's> {
	agents: Query<'w, 's, ReadComponents>,
	items: Res<'w, Assets<Item>>,
	skills: Res<'w, Assets<Skill>>,
}

type ReadComponents = (
	Ref<'static, Slots>,
	Ref<'static, Inventory>,
	Ref<'static, Combos>,
	Ref<'static, Queue>,
);

#[derive(SystemParam)]
pub struct LoadoutWriter<'w, 's, TAssetServer = AssetServer>
where
	TAssetServer: Resource + LoadAsset,
{
	commands: ZyheedaCommands<'w, 's>,
	slots: Query<'w, 's, &'static mut Slots>,
	inventories: Query<'w, 's, &'static mut Inventory>,
	combos: Query<'w, 's, &'static mut Combos>,
	skills: Res<'w, Assets<Skill>>,
	asset_server: ResMut<'w, TAssetServer>,
}
