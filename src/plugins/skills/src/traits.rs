pub(crate) mod advance_combo;
pub(crate) mod flush;
pub(crate) mod is_timed_out;
pub(crate) mod loadout_key;
pub(crate) mod peek_next;
pub(crate) mod peek_next_recursive;
pub(crate) mod skill_builder;
pub(crate) mod skill_state;
pub(crate) mod spawn_skill_behavior;
pub(crate) mod user_input;
pub(crate) mod visualize_item;
pub(crate) mod write_item;

use crate::{
	behaviors::SkillCaster,
	components::SkillTarget,
	skills::{AnimationStrategy, RunSkillBehavior, Skill},
};
use common::{
	tools::{action_key::slot::SlotKey, item_type::ItemType},
	traits::{key_mappings::TryGetAction, state_duration::UpdatedStates},
	zyheeda_commands::ZyheedaCommands,
};

pub(crate) trait Enqueue<TItem> {
	fn enqueue(&mut self, item: TItem);
}

pub(crate) trait Flush {
	fn flush(&mut self);
}

pub(crate) trait IterHoldingMut<TItem> {
	fn iter_holding_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut TItem>
	where
		TItem: 'a;
}

pub(crate) trait IterAddedMut {
	type TItem;

	fn added_none(&self) -> bool;
	fn iter_added_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut Self::TItem>
	where
		Self::TItem: 'a;
}

pub(crate) trait ReleaseSkill {
	/// Release a skill. What this means depends on the actual skill behavior.
	///
	/// For instance:
	/// - A shield is dropped.
	/// - A pistol shoots a projectile
	fn release_skill(&mut self);
}

pub(crate) trait GetActiveSkill<TSkillState> {
	type TActive<'a>: GetSkillBehavior + GetAnimationStrategy + UpdatedStates<TSkillState>
	where
		Self: 'a;

	fn get_active(&mut self) -> Option<Self::TActive<'_>>;
	fn clear_active(&mut self);
}

pub(crate) trait AdvanceCombo {
	fn advance_combo(&mut self, trigger: SlotKey, item_type: &ItemType) -> Option<Skill>;
}

pub(crate) trait SetNextCombo<TCombo> {
	fn set_next_combo(&mut self, value: TCombo);
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

pub trait ReKey {
	fn re_key(&mut self, key: SlotKey);
}

pub(crate) trait GetAnimationStrategy {
	fn animation_strategy(&self) -> AnimationStrategy;
}

pub(crate) trait GetSkillBehavior {
	fn behavior(&self) -> (SlotKey, RunSkillBehavior);
}

pub trait InputState<TMap, TOutput>
where
	TMap: TryGetAction<TOutput>,
{
	fn just_pressed_slots(&self, map: &TMap) -> Vec<TOutput>;
	fn pressed_slots(&self, map: &TMap) -> Vec<TOutput>;
}

pub trait Schedule<TBehavior> {
	fn schedule(&mut self, slot_key: SlotKey, behavior: TBehavior);
}

pub(crate) trait Execute<TEffects, TSkillBehavior> {
	fn execute(
		&mut self,
		commands: &mut ZyheedaCommands,
		caster: &SkillCaster,
		target: &SkillTarget,
	);
}
