pub(crate) mod advance_combo;
pub(crate) mod bevy_input;
pub(crate) mod flush;
pub(crate) mod get_skill_animation;
pub(crate) mod peek_next;
pub(crate) mod skill_builder;
pub(crate) mod skill_state;
pub(crate) mod spawn_skill_behavior;
pub(crate) mod state;
pub(crate) mod swap_commands;

use crate::{
	behaviors::{SkillCaster, Target},
	components::skill_spawners::SkillSpawners,
	item::item_type::SkillItemType,
	skills::{Animate, RunSkillBehavior, Skill, SkillAnimation},
	slot_key::SlotKey,
};
use common::traits::{load_asset::Path, map_value::TryMapBackwards, state_duration::StateUpdate};
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

pub(crate) trait GetActiveSkill<TAnimation, TSkillState: Clone> {
	fn get_active(
		&mut self,
	) -> Option<impl GetSkillBehavior + GetAnimation<TAnimation> + StateUpdate<TSkillState>>;
	fn clear_active(&mut self);
}

pub trait IsTimedOut {
	fn is_timed_out(&self) -> bool;
}

pub trait PeekNext<TNext> {
	fn peek_next(&self, trigger: &SlotKey, item_type: &SkillItemType) -> Option<TNext>;
}

pub(crate) trait AdvanceCombo {
	fn advance_combo(&mut self, trigger: &SlotKey, item_type: &SkillItemType) -> Option<Skill>;
}

pub(crate) trait SetNextCombo<TCombo> {
	fn set_next_combo(&mut self, value: TCombo);
}

pub type Combo<'a> = Vec<(Vec<SlotKey>, &'a Skill)>;

pub trait GetCombosOrdered {
	fn combos_ordered(&self) -> impl Iterator<Item = Combo>;
}

pub trait GetNode<'a, TKey> {
	type TNode;
	fn node(&'a self, key: &TKey) -> Option<Self::TNode>;
}

pub trait GetNodeMut<'a, TKey> {
	type TNode;
	fn node_mut(&'a mut self, key: &TKey) -> Option<Self::TNode>;
}

pub trait RootKeys {
	type TItem;
	fn root_keys(&self) -> impl Iterator<Item = Self::TItem>;
}

pub trait FollowupKeys {
	type TItem;
	fn followup_keys(&self) -> impl Iterator<Item = Self::TItem>;
}

pub trait Insert<T> {
	fn insert(&mut self, value: T);
}

pub trait ReKey<TKey> {
	fn re_key(&mut self, key: TKey);
}

pub trait UpdateConfig<TKey, TValue> {
	fn update_config(&mut self, key: &TKey, value: TValue);
}

pub(crate) trait GetAnimation<TAnimation> {
	fn animate(&self) -> Animate<TAnimation>;
}

pub trait GetStaticSkillBehavior {
	fn behavior() -> RunSkillBehavior;
}

pub(crate) trait GetSkillBehavior {
	fn behavior(&self) -> (SlotKey, RunSkillBehavior);
}

pub trait InputState<TMap: TryMapBackwards<TKey, SlotKey>, TKey: Eq + Hash> {
	fn just_pressed_slots(&self, map: &TMap) -> Vec<SlotKey>;
	fn pressed_slots(&self, map: &TMap) -> Vec<SlotKey>;
	fn just_released_slots(&self, map: &TMap) -> Vec<SlotKey>;
}

pub trait Schedule<TBehavior> {
	fn schedule(&mut self, slot_key: SlotKey, behavior: TBehavior);
}

pub(crate) trait Execute<TCommands> {
	type TError;

	fn execute(
		&mut self,
		commands: &mut TCommands,
		caster: &SkillCaster,
		spawners: &SkillSpawners,
		target: &Target,
	) -> Result<(), Self::TError>;
}

pub trait ShouldEnqueue {
	fn should_enqueue(&self) -> bool;
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
