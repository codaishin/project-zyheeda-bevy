use crate::types::BoneName;
use bevy::prelude::{Component, Entity};
use common::{
	components::SlotKey,
	skill::{Skill, SkillComboTree},
	tools::UnitsPerSecond,
};
use std::{collections::HashMap, fmt::Debug, time::Duration};

#[derive(Component, Clone)]
pub enum VoidSpherePart {
	Core,
	RingA(UnitsPerSecond),
	RingB(UnitsPerSecond),
}

#[derive(Component, Default)]
pub struct Animator {
	pub animation_player_id: Option<Entity>,
}

#[derive(Component)]
pub struct Mark<T>(pub T);

#[derive(Component, Clone, PartialEq, Debug)]
pub struct SlotBones(pub HashMap<SlotKey, &'static BoneName>);

#[derive(Component, PartialEq, Debug)]
pub enum Schedule {
	Enqueue((SlotKey, Skill)),
	Override((SlotKey, Skill)),
	StopAimAfter(Duration),
	UpdateTarget,
}

#[derive(Component, Clone)]
pub struct ComboTreeTemplate<TNext>(pub HashMap<SlotKey, SkillComboTree<TNext>>);

#[derive(Component, PartialEq, Debug)]
pub struct ComboTreeRunning<TNext>(pub HashMap<SlotKey, SkillComboTree<TNext>>);

#[derive(Component)]
pub struct Dummy;
