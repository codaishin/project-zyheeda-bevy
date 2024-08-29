pub mod shoot_hand_gun;
pub mod skill_data;

use crate::{
	behaviors::Behavior,
	items::{slot_key::SlotKey, ItemType},
	traits::{Matches, Prime},
};
use animations::animation::Animation;
use behaviors::components::{Contact, Projection};
use bevy::{
	asset::{Asset, Handle},
	math::{Dir3, Ray3d, Vec3},
	prelude::Image,
	reflect::TypePath,
	utils::default,
};
use common::resources::ColliderInfo;
use std::{
	collections::HashSet,
	fmt::{Display, Formatter, Result},
	time::Duration,
};

#[derive(PartialEq, Debug, Clone)]
pub struct SkillAnimation {
	pub(crate) left: Animation,
	pub(crate) right: Animation,
}

#[derive(PartialEq, Debug, Default, Clone)]
pub enum Animate<TAnimation> {
	#[default]
	Ignore,
	None,
	Some(TAnimation),
}

#[derive(PartialEq, Debug, Default, Clone, TypePath, Asset)]
pub struct Skill {
	pub name: String,
	pub active: Duration,
	pub animate: Animate<SkillAnimation>,
	pub behavior: SkillBehavior,
	pub is_usable_with: HashSet<ItemType>,
	pub icon: Option<Handle<Image>>,
}

impl Display for Skill {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self.name.as_str() {
			"" => write!(f, "Skill(<no name>)"),
			name => write!(f, "Skill({})", name),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SelectInfo<T> {
	pub ray: Ray3d,
	pub collision_info: Option<ColliderInfo<T>>,
}

impl<T> Default for SelectInfo<T> {
	fn default() -> Self {
		Self {
			ray: Ray3d {
				origin: Vec3::ZERO,
				direction: Dir3::NEG_Z,
			},
			collision_info: None,
		}
	}
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum Activation {
	#[default]
	Waiting,
	Primed,
	ActiveAfter(Duration),
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct QueuedSkill {
	pub skill: Skill,
	pub slot_key: SlotKey,
	pub mode: Activation,
}

impl Prime for QueuedSkill {
	fn prime(&mut self) {
		if self.mode != Activation::Waiting {
			return;
		}
		self.mode = Activation::Primed;
	}
}

impl Matches<SlotKey> for QueuedSkill {
	fn matches(&self, slot_key: &SlotKey) -> bool {
		&self.slot_key == slot_key
	}
}

#[cfg(test)]
mod test_queued {
	use super::*;
	use bevy::utils::default;

	#[test]
	fn prime_skill() {
		let mut queued = QueuedSkill {
			skill: Skill::default(),
			mode: Activation::Waiting,
			..default()
		};
		queued.prime();

		assert_eq!(Activation::Primed, queued.mode);
	}

	#[test]
	fn do_not_prime_active() {
		let mut queued = QueuedSkill {
			skill: Skill::default(),
			mode: Activation::ActiveAfter(Duration::from_millis(123)),
			..default()
		};
		queued.prime();

		assert_eq!(
			Activation::ActiveAfter(Duration::from_millis(123)),
			queued.mode
		);
	}
}

#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash)]
pub(crate) enum SkillState {
	Aim,
	Active,
}

#[derive(PartialEq, Debug, Clone)]
pub enum SkillBehavior {
	OnActive(SkillBehaviors),
	OnAim(SkillBehaviors),
}

impl Default for SkillBehavior {
	fn default() -> Self {
		Self::OnActive(default())
	}
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct SkillBehaviors {
	pub contact: Behavior<Contact>,
	pub projection: Behavior<Projection>,
}
