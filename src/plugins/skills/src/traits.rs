pub(crate) mod bevy_input;
pub(crate) mod game_state;
pub(crate) mod inventory;
pub(crate) mod mouse_hover;
pub(crate) mod player_skills;
pub(crate) mod projectile;
pub(crate) mod skill_combo_next;
pub(crate) mod skill_state;
pub(crate) mod state;
pub(crate) mod sword;
pub(crate) mod tuple_slot_key_item;

use crate::{
	components::SlotKey,
	resources::SlotMap,
	skill::{Queued, Skill, SkillComboTree, SkillExecution, StartBehaviorFn, StopBehaviorFn},
};
use bevy::ecs::{component::Component, system::Query};
use common::{
	components::{Animate, Outdated},
	resources::ColliderInfo,
	traits::state_duration::StateUpdate,
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

pub(crate) trait LastUnchangedMut<TItem> {
	fn last_unchanged_mut<'a>(&'a mut self) -> Option<&'a mut TItem>
	where
		TItem: 'a;
}

pub(crate) trait Prime {
	fn prime(&mut self);
}

pub(crate) trait GetActiveSkill<TAnimationKey: Clone + Copy, TSkillState: Clone> {
	fn get_active(
		&mut self,
	) -> Option<impl Execution + GetAnimation<TAnimationKey> + GetSlots + StateUpdate<TSkillState>>;
	fn clear_active(&mut self);
}

pub(crate) trait ComboNext
where
	Self: Sized,
{
	fn to_vec(&self, trigger_skill: &Skill<Queued>) -> Vec<(SlotKey, SkillComboTree<Self>)>;
}

pub(crate) trait GetAnimation<TAnimationKey: Clone + Copy> {
	fn animate(&self) -> Animate<TAnimationKey>;
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

#[cfg(test)]
pub(crate) mod test_tools {
	use super::*;
	use crate::skill::{Spawner, Target};
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
