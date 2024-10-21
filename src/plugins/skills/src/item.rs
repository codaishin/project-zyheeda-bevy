pub mod item_type;
pub mod visualization;

use crate::skills::Skill;
use items::item::Item;

pub type SkillItem<TSkill = Skill> = Item<TSkill>;
