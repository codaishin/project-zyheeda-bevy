use crate::{
	components::{combos::Combos, queue::Queue},
	resources::skill_item_assets::SkillItemAssets,
};
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam)]
pub struct SkillItemAssetsUsage<'w, 's> {
	pub(crate) skill_item_assets: SkillItemAssets<'w>,
	pub(crate) usage: Query<'w, 's, (&'static Queue, &'static Combos)>,
}
