pub(crate) mod bevy_input;
pub(crate) mod inventory;
pub(crate) mod mouse_hover;
pub(crate) mod projectile;
pub(crate) mod skill_state;
pub(crate) mod state;
pub(crate) mod tuple_slot_key_item;

use crate::{
	components::{slots::Slots, SlotKey},
	resources::SlotMap,
	skills::{Animate, Skill, SkillAnimation, SkillExecution, StartBehaviorFn, StopBehaviorFn},
};
use animations::animation::Animation;
use bevy::ecs::{component::Component, system::Query};
use common::{
	components::Outdated,
	resources::ColliderInfo,
	tools::{Last, This},
	traits::{load_asset::Path, state_duration::StateUpdate},
};
use std::hash::Hash;

pub(crate) trait Enqueue<TItem> {
	fn enqueue(&mut self, item: TItem);
}

pub(crate) trait Flush {
	fn flush(&mut self);
}

pub trait Iter<TItem> {
	fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a TItem>
	where
		TItem: 'a;
}

pub(crate) trait IterMutWithKeys<TKey, TItem> {
	fn iter_mut_with_keys<'a>(
		&'a mut self,
	) -> impl DoubleEndedIterator<Item = (TKey, &'a mut TItem)>
	where
		TItem: 'a;
}

pub(crate) trait IterAddedMut<TItem> {
	fn iter_added_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut TItem>
	where
		TItem: 'a;
}

pub(crate) trait Prime {
	fn prime(&mut self);
}

pub(crate) trait GetActiveSkill<TAnimation, TSkillState: Clone> {
	fn get_active(
		&mut self,
	) -> Option<impl Execution + GetAnimation<TAnimation> + GetSlots + StateUpdate<TSkillState>>;
	fn clear_active(&mut self);
}

pub(crate) trait NextCombo {
	fn next(&mut self, trigger: &SlotKey, slots: &Slots) -> Option<Skill>;
}

pub(crate) trait GetAnimation<TAnimation> {
	fn animate(&self) -> Animate<TAnimation>;
}

pub(crate) trait WithComponent<T: Component + Copy> {
	fn with_component(&self, query: &Query<&T>) -> Option<ColliderInfo<Outdated<T>>>;
}

pub trait GetExecution {
	fn execution() -> SkillExecution;
}

pub(crate) trait GetSlots {
	fn slots(&self) -> Vec<SlotKey>;
}

pub(crate) trait Execution {
	fn get_start(&self) -> Option<StartBehaviorFn>;
	fn get_stop(&self) -> Option<StopBehaviorFn>;
}

pub trait InputState<TKey: Eq + Hash> {
	fn just_pressed_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey>;
	fn pressed_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey>;
	fn just_released_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey>;
}

pub trait ShouldEnqueue {
	fn should_enqueue(&self) -> bool;
}

pub trait SkillTemplate {
	fn skill() -> Skill;
}

#[derive(Clone)]
pub(crate) struct AnimationChainIf {
	pub this: fn() -> Path,
	pub last: fn() -> Path,
	pub then: fn() -> Path,
}

pub(crate) trait GetAnimationSetup {
	fn get_animation() -> SkillAnimation;
	fn get_chains() -> Vec<AnimationChainIf>;
}

pub(crate) trait GetSkillAnimation {
	fn animation() -> SkillAnimation;
}

fn apply_chain<T: GetAnimationSetup>(mut this: This<Animation>, last: Last<Animation>) {
	let chains = T::get_chains();
	let chain = chains
		.iter()
		.find(|chain| this.path == (chain.this)() && last.path == (chain.last)());

	let Some(chain) = chain else {
		return;
	};

	this.path = (chain.then)();
}

impl<T: GetAnimationSetup> GetSkillAnimation for T {
	fn animation() -> SkillAnimation {
		if T::get_chains().is_empty() {
			return T::get_animation();
		}

		let SkillAnimation {
			mut left,
			mut right,
		} = T::get_animation();

		left.update_fn = Some(apply_chain::<T>);
		right.update_fn = Some(apply_chain::<T>);
		SkillAnimation { left, right }
	}
}

#[cfg(test)]
mod test_animation_chain_skill_animation {
	use super::*;
	use animations::animation::PlayMode;
	use mockall::mock;

	mock! {
		_Setup {}
		impl GetAnimationSetup for _Setup {
			fn get_animation() -> SkillAnimation;
			fn get_chains() -> Vec<AnimationChainIf>;
		}
	}

	#[test]
	fn map_left_and_right_animation() {
		let left = Animation::new(Path::from("left"), PlayMode::Repeat);
		let right = Animation::new(Path::from("right"), PlayMode::Repeat);
		let get_animation = Mock_Setup::get_animation_context();
		let get_chains = Mock_Setup::get_chains_context();

		get_animation.expect().return_const(SkillAnimation {
			left: left.clone(),
			right: right.clone(),
		});
		get_chains.expect().return_const(vec![]);

		assert_eq!(SkillAnimation { left, right }, Mock_Setup::animation())
	}

	#[test]
	fn add_apply_chain_func_when_chains_present() {
		let mut left = Animation::new(Path::from("left"), PlayMode::Repeat);
		let mut right = Animation::new(Path::from("right"), PlayMode::Repeat);
		let get_animation = Mock_Setup::get_animation_context();
		let get_chains = Mock_Setup::get_chains_context();

		get_animation.expect().return_const(SkillAnimation {
			left: left.clone(),
			right: right.clone(),
		});
		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from(""),
			this: || Path::from(""),
			then: || Path::from(""),
		}]);

		left.update_fn = Some(apply_chain::<Mock_Setup>);
		right.update_fn = Some(apply_chain::<Mock_Setup>);

		assert_eq!(SkillAnimation { left, right }, Mock_Setup::animation())
	}

	#[test]
	fn apply_chain_combo() {
		let get_chains = Mock_Setup::get_chains_context();

		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from("1"),
			this: || Path::from("2"),
			then: || Path::from("3"),
		}]);

		let mut this = Animation::new(Path::from("2"), PlayMode::Repeat);
		let last = Animation::new(Path::from("1"), PlayMode::Repeat);
		apply_chain::<Mock_Setup>(This(&mut this), Last(&last));

		assert_eq!(Path::from("3"), this.path);
	}

	#[test]
	fn do_not_apply_chain_when_this_mismatch() {
		let get_chains = Mock_Setup::get_chains_context();

		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from("1"),
			this: || Path::from("2"),
			then: || Path::from("3"),
		}]);

		let mut this = Animation::new(Path::from("2 mismatch"), PlayMode::Repeat);
		let last = Animation::new(Path::from("1"), PlayMode::Repeat);
		apply_chain::<Mock_Setup>(This(&mut this), Last(&last));

		assert_eq!(Path::from("2 mismatch"), this.path);
	}

	#[test]
	fn do_not_apply_chain_when_last_mismatch() {
		let get_chains = Mock_Setup::get_chains_context();

		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from("1"),
			this: || Path::from("2"),
			then: || Path::from("3"),
		}]);

		let mut this = Animation::new(Path::from("2"), PlayMode::Repeat);
		let last = Animation::new(Path::from("1 mismatch"), PlayMode::Repeat);
		apply_chain::<Mock_Setup>(This(&mut this), Last(&last));

		assert_eq!(Path::from("2"), this.path);
	}

	#[test]
	fn apply_different_chain() {
		let get_chains = Mock_Setup::get_chains_context();

		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from("d1"),
			this: || Path::from("d2"),
			then: || Path::from("d3"),
		}]);

		let mut this = Animation::new(Path::from("d2"), PlayMode::Repeat);
		let last = Animation::new(Path::from("d1"), PlayMode::Repeat);
		apply_chain::<Mock_Setup>(This(&mut this), Last(&last));

		assert_eq!(Path::from("d3"), this.path);
	}
}

#[cfg(test)]
pub(crate) mod test_tools {
	use super::*;
	use crate::skills::{Spawner, Target};
	use bevy::{ecs::system::Commands, prelude::Entity, transform::components::Transform};

	pub fn run_lazy(
		behavior: SkillExecution,
		agent: Entity,
		agent_transform: Transform,
		spawner: Spawner,
		select_info: Target,
	) -> impl FnMut(Commands) {
		move |mut commands| {
			let Some(run) = behavior.run_fn else {
				return;
			};
			let mut agent = commands.entity(agent);
			run(&mut agent, &agent_transform, &spawner, &select_info);
		}
	}
}
