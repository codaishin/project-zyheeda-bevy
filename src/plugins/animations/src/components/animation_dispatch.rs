mod dto;

use crate::{
	components::animation_dispatch::dto::AnimationDispatchDto,
	traits::{
		AnimationPlayers,
		AnimationPlayersWithoutGraph,
		GetActiveAnimations,
		GetAllActiveAnimations,
	},
};
use bevy::prelude::*;
use common::traits::{
	animation::{
		Animation,
		AnimationPriority,
		ConfigureNewAnimationDispatch,
		SetAnimations,
		StartAnimation,
		StopAnimation,
	},
	register_derived_component::{DerivableFrom, InsertDerivedComponent},
	track::{IsTracking, Track, Untrack},
};
use macros::SavableComponent;
use std::{
	collections::{
		HashSet,
		hash_set::{IntoIter, Iter},
	},
	fmt::Debug,
	hash::Hash,
};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[savable_component(dto = AnimationDispatchDto)]
pub struct AnimationDispatch<TAnimation = Animation>
where
	TAnimation: Eq + Hash,
{
	pub(crate) animation_players: HashSet<Entity>,
	animation_handles: HashSet<Entity>,
	stack: (
		HashSet<TAnimation>,
		HashSet<TAnimation>,
		HashSet<TAnimation>,
	),
}

#[cfg(test)]
impl AnimationDispatch {
	pub(crate) fn to<const N: usize>(entities: [Entity; N]) -> Self {
		let mut dispatch = Self::default();
		for entity in entities {
			dispatch.animation_players.insert(entity);
		}

		dispatch
	}
}

impl<TAnimation> AnimationDispatch<TAnimation>
where
	TAnimation: Eq + Hash,
{
	fn slot_mut<TLayer>(&mut self, layer: TLayer) -> &mut HashSet<TAnimation>
	where
		TLayer: Into<AnimationPriority>,
	{
		match layer.into() {
			AnimationPriority::High => &mut self.stack.0,
			AnimationPriority::Medium => &mut self.stack.1,
			AnimationPriority::Low => &mut self.stack.2,
		}
	}

	fn slot<TLayer>(&self, layer: TLayer) -> &HashSet<TAnimation>
	where
		TLayer: Into<AnimationPriority>,
	{
		match layer.into() {
			AnimationPriority::High => &self.stack.0,
			AnimationPriority::Medium => &self.stack.1,
			AnimationPriority::Low => &self.stack.2,
		}
	}

	fn start_animation<TLayer>(&mut self, layer: TLayer, animation: TAnimation)
	where
		TLayer: Into<AnimationPriority>,
	{
		self.slot_mut(layer).insert(animation);
	}

	fn set_animations<TLayer, TAnimations>(&mut self, layer: TLayer, animations: TAnimations)
	where
		TLayer: Into<AnimationPriority>,
		TAnimations: IntoIterator<Item = TAnimation>,
	{
		*self.slot_mut(layer) = HashSet::from_iter(animations)
	}
}

impl<TAnimation> Default for AnimationDispatch<TAnimation>
where
	TAnimation: Eq + Hash,
{
	fn default() -> Self {
		Self {
			animation_players: default(),
			animation_handles: default(),
			stack: default(),
		}
	}
}

impl<'w, 's, TComponent> DerivableFrom<'w, 's, TComponent> for AnimationDispatch
where
	TComponent: ConfigureNewAnimationDispatch,
{
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::IfNew;

	type TParam = ();

	fn derive_from(_: Entity, component: &TComponent, _: &()) -> Self {
		let mut dispatch = AnimationDispatch::default();
		TComponent::configure_animation_dispatch(component, &mut dispatch);
		dispatch
	}
}

impl Track<AnimationPlayer> for AnimationDispatch {
	fn track(&mut self, entity: Entity, _: &AnimationPlayer) {
		self.animation_players.insert(entity);
	}
}

impl IsTracking<AnimationPlayer> for AnimationDispatch {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.animation_players.contains(entity)
	}
}

impl Untrack<AnimationPlayer> for AnimationDispatch {
	fn untrack(&mut self, entity: &Entity) {
		self.animation_players.remove(entity);
	}
}

impl Track<AnimationGraphHandle> for AnimationDispatch {
	fn track(&mut self, entity: Entity, _: &AnimationGraphHandle) {
		self.animation_handles.insert(entity);
	}
}

impl IsTracking<AnimationGraphHandle> for AnimationDispatch {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.animation_handles.contains(entity)
	}
}

impl Untrack<AnimationGraphHandle> for AnimationDispatch {
	fn untrack(&mut self, entity: &Entity) {
		self.animation_handles.remove(entity);
	}
}

impl AnimationPlayers for AnimationDispatch {
	type TIter = IntoIter<Entity>;

	fn animation_players(&self) -> Self::TIter {
		self.animation_players.clone().into_iter()
	}
}

impl AnimationPlayersWithoutGraph for AnimationDispatch {
	type TIter = std::vec::IntoIter<Entity>;

	fn animation_players_without_graph(&self) -> Self::TIter {
		let entities = self
			.animation_players
			.iter()
			.filter(|e| !self.animation_handles.contains(e))
			.copied()
			.collect::<Vec<_>>();
		entities.into_iter()
	}
}

impl<TAnimation> GetActiveAnimations<TAnimation> for AnimationDispatch<TAnimation>
where
	TAnimation: Clone + Eq + Hash,
{
	type TIter<'a>
		= Iter<'a, TAnimation>
	where
		Self: 'a,
		TAnimation: 'a;

	fn get_active_animations<TPriority>(&self, priority: TPriority) -> Self::TIter<'_>
	where
		TPriority: Into<AnimationPriority>,
	{
		self.slot(priority).iter()
	}
}

impl<TAnimation> GetAllActiveAnimations<TAnimation> for AnimationDispatch<TAnimation>
where
	TAnimation: Clone + Eq + Hash,
{
	type TIter<'a>
		= IterAll<'a, TAnimation>
	where
		Self: 'a,
		TAnimation: 'a;

	fn get_all_active_animations(&self) -> Self::TIter<'_> {
		IterAll(
			self.stack.0.iter(),
			self.stack.1.iter(),
			self.stack.2.iter(),
		)
	}
}

pub struct IterAll<'a, TAnimation>(
	Iter<'a, TAnimation>,
	Iter<'a, TAnimation>,
	Iter<'a, TAnimation>,
);

impl<'a, TAnimation> Iterator for IterAll<'a, TAnimation> {
	type Item = &'a TAnimation;

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

impl StartAnimation for AnimationDispatch {
	fn start_animation<TLayer>(&mut self, layer: TLayer, animation: Animation)
	where
		TLayer: Into<AnimationPriority>,
	{
		self.start_animation(layer, animation);
	}
}

impl SetAnimations for AnimationDispatch {
	fn set_animations<TLayer, TAnimations>(&mut self, layer: TLayer, animations: TAnimations)
	where
		TLayer: Into<AnimationPriority> + 'static,
		TAnimations: IntoIterator<Item = Animation>,
	{
		self.set_animations(layer, animations)
	}
}

impl<TAnimation> StopAnimation for AnimationDispatch<TAnimation>
where
	TAnimation: Eq + Hash,
{
	fn stop_animation<TLayer>(&mut self, layer: TLayer)
	where
		TLayer: Into<AnimationPriority>,
	{
		self.slot_mut(layer).clear();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::default;

	#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
	struct _Animation {
		name: &'static str,
	}

	impl _Animation {
		fn new(name: &'static str) -> Self {
			Self { name }
		}
	}

	struct _Lo;

	impl From<_Lo> for AnimationPriority {
		fn from(_: _Lo) -> Self {
			AnimationPriority::Low
		}
	}
	struct _Me;

	impl From<_Me> for AnimationPriority {
		fn from(_: _Me) -> Self {
			AnimationPriority::Medium
		}
	}

	struct _Hi;

	impl From<_Hi> for AnimationPriority {
		fn from(_: _Hi) -> Self {
			AnimationPriority::High
		}
	}

	#[test]
	fn insert_priorities() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Lo, _Animation::new("low/1"));
		dispatch.start_animation(_Lo, _Animation::new("low/2"));
		dispatch.start_animation(_Me, _Animation::new("medium/1"));
		dispatch.start_animation(_Me, _Animation::new("medium/2"));
		dispatch.start_animation(_Hi, _Animation::new("high/1"));
		dispatch.start_animation(_Hi, _Animation::new("high/2"));

		assert_eq!(
			[
				HashSet::from([(&_Animation::new("low/1")), &_Animation::new("low/2")]),
				HashSet::from([(&_Animation::new("medium/1")), &_Animation::new("medium/2")]),
				HashSet::from([(&_Animation::new("high/1")), &_Animation::new("high/2")]),
			],
			[
				dispatch.get_active_animations(_Lo).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Me).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Hi).collect::<HashSet<_>>(),
			]
		);
	}

	#[test]
	fn stop_animations_low() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Lo, _Animation::new("low"));
		dispatch.start_animation(_Me, _Animation::new("medium"));
		dispatch.start_animation(_Hi, _Animation::new("high"));
		dispatch.stop_animation(_Lo);

		assert_eq!(
			[
				HashSet::from([]),
				HashSet::from([&_Animation::new("medium")]),
				HashSet::from([&_Animation::new("high")]),
			],
			[
				dispatch.get_active_animations(_Lo).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Me).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Hi).collect::<HashSet<_>>(),
			]
		);
	}

	#[test]
	fn stop_animations_medium() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Lo, _Animation::new("low"));
		dispatch.start_animation(_Me, _Animation::new("medium"));
		dispatch.start_animation(_Hi, _Animation::new("high"));
		dispatch.stop_animation(_Me);

		assert_eq!(
			[
				HashSet::from([&_Animation::new("low")]),
				HashSet::from([]),
				HashSet::from([&_Animation::new("high")]),
			],
			[
				dispatch.get_active_animations(_Lo).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Me).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Hi).collect::<HashSet<_>>(),
			]
		);
	}
	#[test]
	fn stop_animations_high() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Lo, _Animation::new("low"));
		dispatch.start_animation(_Me, _Animation::new("medium"));
		dispatch.start_animation(_Hi, _Animation::new("high"));
		dispatch.stop_animation(_Hi);

		assert_eq!(
			[
				HashSet::from([&_Animation::new("low")]),
				HashSet::from([&_Animation::new("medium")]),
				HashSet::from([]),
			],
			[
				dispatch.get_active_animations(_Lo).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Me).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Hi).collect::<HashSet<_>>(),
			]
		);
	}

	#[test]
	fn override_animations_low() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Lo, _Animation::new("low"));
		dispatch.start_animation(_Me, _Animation::new("medium"));
		dispatch.start_animation(_Hi, _Animation::new("high"));
		dispatch.set_animations(
			_Lo,
			[_Animation::new("override 1"), _Animation::new("override 2")],
		);

		assert_eq!(
			[
				HashSet::from([
					&_Animation::new("override 1"),
					&_Animation::new("override 2")
				]),
				HashSet::from([&_Animation::new("medium")]),
				HashSet::from([&_Animation::new("high")]),
			],
			[
				dispatch.get_active_animations(_Lo).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Me).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Hi).collect::<HashSet<_>>(),
			]
		);
	}

	#[test]
	fn override_animations_medium() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Lo, _Animation::new("low"));
		dispatch.start_animation(_Me, _Animation::new("medium"));
		dispatch.start_animation(_Hi, _Animation::new("high"));
		dispatch.set_animations(
			_Me,
			[_Animation::new("override 1"), _Animation::new("override 2")],
		);

		assert_eq!(
			[
				HashSet::from([&_Animation::new("low")]),
				HashSet::from([
					&_Animation::new("override 1"),
					&_Animation::new("override 2")
				]),
				HashSet::from([&_Animation::new("high")]),
			],
			[
				dispatch.get_active_animations(_Lo).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Me).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Hi).collect::<HashSet<_>>(),
			]
		);
	}
	#[test]
	fn override_animations_high() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Lo, _Animation::new("low"));
		dispatch.start_animation(_Me, _Animation::new("medium"));
		dispatch.start_animation(_Hi, _Animation::new("high"));
		dispatch.set_animations(
			_Hi,
			[_Animation::new("override 1"), _Animation::new("override 2")],
		);

		assert_eq!(
			[
				HashSet::from([&_Animation::new("low")]),
				HashSet::from([&_Animation::new("medium")]),
				HashSet::from([
					&_Animation::new("override 1"),
					&_Animation::new("override 2")
				]),
			],
			[
				dispatch.get_active_animations(_Lo).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Me).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Hi).collect::<HashSet<_>>(),
			]
		);
	}

	fn as_track<TComponent>(
		tracker: &mut (impl Track<TComponent> + IsTracking<TComponent> + Untrack<TComponent>),
	) -> &mut (impl Track<TComponent> + IsTracking<TComponent> + Untrack<TComponent>)
	where
		AnimationDispatch: Track<TComponent> + IsTracking<TComponent> + Untrack<TComponent>,
	{
		tracker
	}

	#[test]
	fn track_animation_player() {
		let dispatch = &mut AnimationDispatch::default();
		as_track::<AnimationPlayer>(dispatch)
			.track(Entity::from_raw(1), &AnimationPlayer::default());
		as_track::<AnimationPlayer>(dispatch)
			.track(Entity::from_raw(2), &AnimationPlayer::default());

		assert_eq!(
			HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			dispatch.animation_players
		)
	}

	#[test]
	fn untrack_animation_player() {
		let dispatch = &mut AnimationDispatch {
			animation_players: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};
		as_track::<AnimationPlayer>(dispatch).untrack(&Entity::from_raw(1));

		assert_eq!(
			HashSet::from([Entity::from_raw(2)]),
			dispatch.animation_players
		)
	}

	#[test]
	fn is_tracking_animation_player() {
		let dispatch = &mut AnimationDispatch {
			animation_players: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};

		assert_eq!(
			[true, false],
			[
				as_track::<AnimationPlayer>(dispatch).is_tracking(&Entity::from_raw(2)),
				as_track::<AnimationPlayer>(dispatch).is_tracking(&Entity::from_raw(3)),
			]
		)
	}

	#[test]
	fn track_animation_graph() {
		let dispatch = &mut AnimationDispatch::default();
		as_track::<AnimationGraphHandle>(dispatch)
			.track(Entity::from_raw(1), &AnimationGraphHandle::default());
		as_track::<AnimationGraphHandle>(dispatch)
			.track(Entity::from_raw(2), &AnimationGraphHandle::default());

		assert_eq!(
			HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			dispatch.animation_handles
		)
	}

	#[test]
	fn untrack_animation_graph() {
		let dispatch = &mut AnimationDispatch {
			animation_handles: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};
		as_track::<AnimationGraphHandle>(dispatch).untrack(&Entity::from_raw(1));

		assert_eq!(
			HashSet::from([Entity::from_raw(2)]),
			dispatch.animation_handles
		)
	}

	#[test]
	fn is_tracking_animation_graph() {
		let dispatch = &mut AnimationDispatch {
			animation_handles: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};

		assert_eq!(
			[true, false],
			[
				as_track::<AnimationGraphHandle>(dispatch).is_tracking(&Entity::from_raw(2)),
				as_track::<AnimationGraphHandle>(dispatch).is_tracking(&Entity::from_raw(3)),
			]
		)
	}

	#[test]
	fn iterate_animation_players() {
		let dispatch = AnimationDispatch {
			animation_players: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};

		assert_eq!(
			HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			dispatch.animation_players().collect::<HashSet<_>>(),
		)
	}

	#[test]
	fn iterate_animation_players_without_transitions() {
		let dispatch = AnimationDispatch {
			animation_players: HashSet::from([
				Entity::from_raw(1),
				Entity::from_raw(2),
				Entity::from_raw(3),
			]),
			animation_handles: HashSet::from([Entity::from_raw(2)]),
			..default()
		};

		assert_eq!(
			HashSet::from([Entity::from_raw(1), Entity::from_raw(3)]),
			dispatch
				.animation_players_without_graph()
				.collect::<HashSet<_>>(),
		)
	}

	#[test]
	fn iter_all() {
		let dispatch = AnimationDispatch {
			animation_players: default(),
			animation_handles: default(),
			stack: (
				HashSet::from([1, 2, 3]),
				HashSet::from([4, 5, 6]),
				HashSet::from([7, 8, 9]),
			),
		};

		assert_eq!(
			HashSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9]),
			dispatch
				.get_all_active_animations()
				.copied()
				.collect::<HashSet<_>>()
		)
	}
}
