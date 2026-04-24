use crate::{
	components::{
		current_forward_pitch::CurrentForwardPitch,
		current_movement_direction::CurrentMovementDirection,
	},
	traits::{GetAllActiveAnimations, YoungestToOldestActiveAnimations},
};
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::traits::handles_animations::{AnimationKey, AnimationPriority};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, iter::Rev};
use zyheeda_core::prelude::*;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(CurrentMovementDirection, CurrentForwardPitch)]
#[savable_component(id = "animation dispatch")]
pub struct AnimationDispatch {
	priorities: (
		OrderedSet<AnimationKey>,
		OrderedSet<AnimationKey>,
		OrderedSet<AnimationKey>,
	),
}

impl AnimationDispatch {
	pub(crate) fn slot_mut<TLayer>(&mut self, layer: TLayer) -> &mut OrderedSet<AnimationKey>
	where
		TLayer: Into<AnimationPriority>,
	{
		match layer.into() {
			AnimationPriority::High => &mut self.priorities.0,
			AnimationPriority::Medium => &mut self.priorities.1,
			AnimationPriority::Low => &mut self.priorities.2,
		}
	}

	pub(crate) fn slot<TLayer>(&self, layer: TLayer) -> &OrderedSet<AnimationKey>
	where
		TLayer: Into<AnimationPriority>,
	{
		match layer.into() {
			AnimationPriority::High => &self.priorities.0,
			AnimationPriority::Medium => &self.priorities.1,
			AnimationPriority::Low => &self.priorities.2,
		}
	}
}

impl Default for AnimationDispatch {
	fn default() -> Self {
		Self {
			priorities: default(),
		}
	}
}

impl YoungestToOldestActiveAnimations for AnimationDispatch {
	type TIter<'a>
		= Rev<UniqueIter<'a, AnimationKey>>
	where
		Self: 'a;

	fn youngest_to_oldest_active_animations<TPriority>(
		&self,
		priority: TPriority,
	) -> Self::TIter<'_>
	where
		TPriority: Into<AnimationPriority>,
	{
		self.slot(priority).iter().rev()
	}
}

impl GetAllActiveAnimations for AnimationDispatch {
	type TIter<'a>
		= IterAllAnimations<'a>
	where
		Self: 'a;

	fn get_all_active_animations(&self) -> Self::TIter<'_> {
		IterAllAnimations(
			self.priorities.0.iter(),
			self.priorities.1.iter(),
			self.priorities.2.iter(),
		)
	}
}

pub struct IterAllAnimations<'a>(
	UniqueIter<'a, AnimationKey>,
	UniqueIter<'a, AnimationKey>,
	UniqueIter<'a, AnimationKey>,
);

impl<'a> Iterator for IterAllAnimations<'a> {
	type Item = &'a AnimationKey;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(next) = self.0.next() {
			return Some(next);
		}
		if let Some(next) = self.1.next() {
			return Some(next);
		}
		if let Some(next) = self.2.next() {
			return Some(next);
		}

		None
	}
}

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = AnimationPlayerOf)]
pub(crate) struct AnimationPlayers(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = AnimationPlayers)]
pub(crate) struct AnimationPlayerOf(pub(crate) Entity);

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = AnimationGraphOf)]
pub(crate) struct AnimationGraphs(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = AnimationGraphs)]
pub(crate) struct AnimationGraphOf(pub(crate) Entity);
