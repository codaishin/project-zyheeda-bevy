pub mod shoot_hand_gun;

use crate::{
	items::{ItemType, SlotKey},
	traits::Prime,
};
use animations::animation::Animation;
use bevy::{
	ecs::system::EntityCommands,
	math::{primitives::Direction3d, Ray3d, Vec3},
	transform::components::{GlobalTransform, Transform},
};
use common::{components::Outdated, resources::ColliderInfo, traits::load_asset::Path};
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

#[derive(PartialEq, Debug, Default, Clone)]
pub struct Skill<TData = ()> {
	pub name: &'static str,
	pub data: TData,
	pub cast: Cast,
	pub animate: Animate<SkillAnimation>,
	pub execution: SkillExecution,
	pub is_usable_with: HashSet<ItemType>,
	pub icon: Option<fn() -> Path>,
}

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub struct Cast {
	pub pre: Duration,
	pub active: Duration,
	pub after: Duration,
}

impl<TData> Display for Skill<TData> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self.name {
			"" => write!(f, "Skill(<no name>)"),
			name => write!(f, "Skill({})", name),
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct SelectInfo<T> {
	pub ray: Ray3d,
	pub collision_info: Option<ColliderInfo<T>>,
}

impl<T> Default for SelectInfo<T> {
	fn default() -> Self {
		Self {
			ray: Ray3d {
				origin: Vec3::ZERO,
				direction: Direction3d::NEG_Z,
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
pub struct Queued {
	pub slot_key: SlotKey,
	pub mode: Activation,
}

impl Skill {
	pub fn with<TData: Clone>(self, data: TData) -> Skill<TData> {
		Skill {
			data,
			name: self.name,
			cast: self.cast,
			animate: self.animate,
			execution: self.execution,
			is_usable_with: self.is_usable_with,
			icon: self.icon,
		}
	}
}

impl<TSrc> Skill<TSrc> {
	pub fn map_data<TDst>(self, map: fn(TSrc) -> TDst) -> Skill<TDst> {
		Skill {
			name: self.name,
			data: map(self.data),
			cast: self.cast,
			animate: self.animate,
			execution: self.execution,
			is_usable_with: self.is_usable_with,
			icon: self.icon,
		}
	}
}

impl Prime for Skill<Queued> {
	fn prime(&mut self) {
		if self.data.mode != Activation::Waiting {
			return;
		}
		self.data.mode = Activation::Primed;
	}
}

#[cfg(test)]
mod test_skill {
	use super::*;
	use bevy::utils::default;

	#[test]
	fn prime_skill() {
		let mut skill = Skill {
			data: Queued {
				mode: Activation::Waiting,
				..default()
			},
			..default()
		};
		skill.prime();

		assert_eq!(Activation::Primed, skill.data.mode);
	}

	#[test]
	fn do_not_prime_active() {
		let mut skill = Skill {
			data: Queued {
				mode: Activation::ActiveAfter(Duration::from_millis(123)),
				..default()
			},
			..default()
		};
		skill.prime();

		assert_eq!(
			Activation::ActiveAfter(Duration::from_millis(123)),
			skill.data.mode
		);
	}
}

#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash)]
pub(crate) enum SkillState {
	Aim,
	PreCast,
	Active,
	AfterCast,
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Spawner(pub GlobalTransform);

pub type Target = SelectInfo<Outdated<GlobalTransform>>;
pub type TransformFN = fn(&mut Transform, &Spawner, &Target);
pub type StartBehaviorFn = fn(&mut EntityCommands, &Transform, &Spawner, &Target);
pub type StopBehaviorFn = fn(&mut EntityCommands);

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub struct SkillExecution {
	pub run_fn: Option<StartBehaviorFn>,
	pub stop_fn: Option<StopBehaviorFn>,
}
