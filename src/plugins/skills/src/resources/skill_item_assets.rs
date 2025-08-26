use crate::{item::Item, skills::Skill};
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam)]
pub struct SkillItemAssets<'w> {
	pub(crate) items: Res<'w, Assets<Item>>,
	pub(crate) skills: Res<'w, Assets<Skill>>,
}
