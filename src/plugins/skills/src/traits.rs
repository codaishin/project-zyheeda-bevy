pub(crate) mod advance_combo;
pub(crate) mod flush;
pub(crate) mod follow_up_keys;
pub(crate) mod is_timed_out;
pub(crate) mod loadout_key;
pub(crate) mod peek_next;
pub(crate) mod peek_next_recursive;
pub(crate) mod skill_builder;
pub(crate) mod skill_state;
pub(crate) mod spawn_skill_behavior;
pub(crate) mod state;
pub(crate) mod user_input;
pub(crate) mod write_item;

use crate::{
	behaviors::SkillCaster,
	components::{SkillTarget, skill_spawners::SkillSpawners},
	skills::{AnimationStrategy, RunSkillBehavior, Skill},
};
use common::{
	tools::{action_key::slot::SlotKey, item_type::ItemType},
	traits::{key_mappings::TryGetKey, state_duration::StateUpdate},
};
use std::hash::Hash;

pub(crate) trait Enqueue<TItem> {
	fn enqueue(&mut self, item: TItem);
}

pub(crate) trait Matches<T> {
	fn matches(&self, value: &T) -> bool;
}

pub(crate) trait Flush {
	fn flush(&mut self);
}

pub(crate) trait IterMut<TItem> {
	fn iter_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut TItem>
	where
		TItem: 'a;
}

pub(crate) trait IterAddedMut<TItem> {
	fn added_none(&self) -> bool;
	fn iter_added_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut TItem>
	where
		TItem: 'a;
}

pub(crate) trait Prime {
	fn prime(&mut self);
}

pub(crate) trait GetActiveSkill<TSkillState: Clone> {
	fn get_active(
		&mut self,
	) -> Option<impl GetSkillBehavior + GetAnimationStrategy + StateUpdate<TSkillState>>;
	fn clear_active(&mut self);
}

pub(crate) trait AdvanceCombo {
	fn advance_combo(&mut self, trigger: &SlotKey, item_type: &ItemType) -> Option<Skill>;
}

pub(crate) trait SetNextCombo<TCombo> {
	fn set_next_combo(&mut self, value: TCombo);
}

pub trait GetNode<TKey> {
	type TNode<'a>
	where
		Self: 'a;
	fn node<'a>(&'a self, key: &TKey) -> Option<Self::TNode<'a>>;
}

pub trait GetNodeMut<TKey> {
	type TNode<'a>
	where
		Self: 'a;
	fn node_mut<'a>(&'a mut self, key: &TKey) -> Option<Self::TNode<'a>>;
}

pub trait Insert<T> {
	fn insert(&mut self, value: T);
}

pub trait ReKey<TKey> {
	fn re_key(&mut self, key: TKey);
}

pub(crate) trait GetAnimationStrategy {
	fn animation_strategy(&self) -> AnimationStrategy;
}

pub trait GetStaticSkillBehavior {
	fn behavior() -> RunSkillBehavior;
}

pub(crate) trait GetSkillBehavior {
	fn behavior(&self) -> (SlotKey, RunSkillBehavior);
}

pub trait InputState<TMap, TKey>
where
	TMap: TryGetKey<TKey, SlotKey>,
	TKey: Eq + Hash,
{
	fn just_pressed_slots(&self, map: &TMap) -> Vec<SlotKey>;
	fn pressed_slots(&self, map: &TMap) -> Vec<SlotKey>;
	fn just_released_slots(&self, map: &TMap) -> Vec<SlotKey>;
}

pub trait Schedule<TBehavior> {
	fn schedule(&mut self, slot_key: SlotKey, behavior: TBehavior);
}

pub(crate) trait Execute<TCommands, TLifetimes, TEffects, TSkillBehavior> {
	type TError;

	fn execute(
		&mut self,
		commands: &mut TCommands,
		caster: &SkillCaster,
		spawners: &SkillSpawners,
		target: &SkillTarget,
	) -> Result<(), Self::TError>;
}

pub trait ShouldEnqueue {
	fn should_enqueue(&self) -> bool;
}
