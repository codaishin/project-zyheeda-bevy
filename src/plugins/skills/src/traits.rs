pub(crate) mod advance_combo;
pub(crate) mod bevy_input;
pub(crate) mod flush;
pub(crate) mod force_shield;
pub(crate) mod get_skill_animation;
pub(crate) mod gravity_well;
pub(crate) mod inventory;
pub(crate) mod peek_next;
pub(crate) mod projectile;
pub(crate) mod run_skill;
pub(crate) mod skill_state;
pub(crate) mod state;
pub(crate) mod tuple_slot_key_item;

use crate::{
	components::slots::Slots,
	items::slot_key::SlotKey,
	resources::SlotMap,
	skills::{
		Animate,
		OnSkillStop,
		Skill,
		SkillAnimation,
		SkillBehavior,
		SkillCaster,
		SkillSpawner,
		StartBehaviorFn,
		Target,
	},
};
use bevy::ecs::{bundle::Bundle, system::Commands};
use common::traits::{load_asset::Path, state_duration::StateUpdate};
use std::hash::Hash;

pub(crate) trait Enqueue<TItem> {
	fn enqueue(&mut self, item: TItem);
}

pub(crate) trait SkillBundleConfig {
	const STOPPABLE: bool;

	fn new_skill_bundle(
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> impl Bundle;
}

pub(crate) trait RunSkill {
	fn run_skill(
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> OnSkillStop;
}

pub(crate) trait Flush {
	fn flush(&mut self);
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
	) -> Option<impl GetSkillBehavior + GetAnimation<TAnimation> + StateUpdate<TSkillState>>;
	fn clear_active(&mut self);
}

pub trait IsLingering {
	fn is_lingering(&self) -> bool;
}

pub trait PeekNext<TNext> {
	fn peek_next(&self, trigger: &SlotKey, slots: &Slots) -> Option<TNext>;
}

pub(crate) trait AdvanceCombo {
	fn advance(&mut self, trigger: &SlotKey, slots: &Slots) -> Option<Skill>;
}

pub(crate) trait SetNextCombo<TCombo> {
	fn set_next_combo(&mut self, value: TCombo);
}

type Combo<'a> = Vec<(&'a SlotKey, &'a Skill)>;

pub trait GetCombos {
	fn combos(&self) -> Vec<Combo>;
}

pub(crate) trait GetAnimation<TAnimation> {
	fn animate(&self) -> Animate<TAnimation>;
}

pub trait GetStaticSkillBehavior {
	fn behavior() -> SkillBehavior;
}

pub(crate) trait GetSkillBehavior {
	fn behavior(&self) -> SkillBehavior;
}

pub trait InputState<TKey: Eq + Hash> {
	fn just_pressed_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey>;
	fn pressed_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey>;
	fn just_released_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey>;
}

pub trait Schedule {
	fn schedule(&mut self, start: StartBehaviorFn);
}

pub trait Execute {
	fn execute(
		&mut self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	);
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
