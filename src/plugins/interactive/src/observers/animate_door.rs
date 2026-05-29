use crate::components::{door::Door, interactive_state::IsActive};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::TryGetContextMut,
	handles_animations::{ActiveAnimationsMut, AnimationKey, AnimationPriority, Animations},
};
use zyheeda_core::collections::ordered::OrderedSet;

impl Door {
	pub(crate) fn animate_open<TAnimations>(
		on_add: On<Add, IsActive>,
		doors: Query<(), With<Self>>,
		mut animations: StaticSystemParam<TAnimations>,
	) where
		TAnimations: for<'c> TryGetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>,
	{
		let entity = on_add.entity;

		if !doors.contains(entity) {
			return;
		};

		let key = Animations { entity };
		let Some(mut animations) = TAnimations::try_get_context_mut(&mut animations, key) else {
			return;
		};

		*animations.active_animations_mut(OpenClose) = OrderedSet::from([AnimationKey::Open]);
	}

	pub(crate) fn animate_close<TAnimations>(
		on_insert: On<Remove, IsActive>,
		doors: Query<(), With<Self>>,
		mut animations: StaticSystemParam<TAnimations>,
	) where
		TAnimations: for<'c> TryGetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>,
	{
		let entity = on_insert.entity;

		if !doors.contains(entity) {
			return;
		};

		let key = Animations { entity };
		let Some(mut animations) = TAnimations::try_get_context_mut(&mut animations, key) else {
			return;
		};

		*animations.active_animations_mut(OpenClose) = OrderedSet::from([AnimationKey::Close]);
	}
}

struct OpenClose;

impl From<OpenClose> for AnimationPriority {
	fn from(_: OpenClose) -> Self {
		Self::High
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use common::traits::handles_animations::{ActiveAnimations, AnimationKey, AnimationPriority};
	use std::collections::HashMap;
	use testing::SingleThreadedApp;
	use zyheeda_core::collections::ordered::OrderedSet;

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Animations(HashMap<AnimationPriority, OrderedSet<AnimationKey>>);

	impl ActiveAnimations for _Animations {
		fn active_animations<TLayer>(&self, layer: TLayer) -> &OrderedSet<AnimationKey>
		where
			TLayer: Into<AnimationPriority>,
		{
			self.0
				.get(&layer.into())
				.unwrap_or(const { &OrderedSet::EMPTY })
		}
	}

	impl ActiveAnimationsMut for _Animations {
		fn active_animations_mut<TLayer>(&mut self, layer: TLayer) -> &mut OrderedSet<AnimationKey>
		where
			TLayer: Into<AnimationPriority>,
		{
			self.0.entry(layer.into()).or_default()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(Door::animate_open::<Query<&mut _Animations>>);
		app.add_observer(Door::animate_close::<Query<&mut _Animations>>);

		app
	}

	mod open {
		use super::*;

		#[test]
		fn play_only_open() {
			let mut app = setup();
			let mut entity = app.world_mut().spawn((
				Door,
				_Animations(HashMap::from([(
					AnimationPriority::High,
					OrderedSet::from([AnimationKey::Close, AnimationKey::Idle]),
				)])),
			));

			entity.insert(IsActive);

			assert_eq!(
				Some(&_Animations(HashMap::from([(
					AnimationPriority::High,
					OrderedSet::from([AnimationKey::Open])
				)]))),
				entity.get::<_Animations>(),
			);
		}

		#[test]
		fn do_nothing_when_door_missing() {
			let mut app = setup();
			let mut entity = app.world_mut().spawn(_Animations::default());

			entity.insert(IsActive);

			assert_eq!(Some(&_Animations::default()), entity.get::<_Animations>(),);
		}

		#[test]
		fn act_only_once() {
			let mut app = setup();
			let mut entity = app.world_mut().spawn((Door, _Animations::default()));

			entity.insert(IsActive);
			let mut animations = entity.get_mut::<_Animations>().unwrap();
			animations
				.0
				.insert(AnimationPriority::High, OrderedSet::from([]));
			entity.insert(IsActive);

			assert_eq!(
				Some(&_Animations(HashMap::from([(
					AnimationPriority::High,
					OrderedSet::from([]),
				)]))),
				entity.get::<_Animations>(),
			);
		}
	}

	mod close {
		use super::*;

		#[test]
		fn play_only_close() {
			let mut app = setup();
			let mut entity = app.world_mut().spawn((
				IsActive,
				Door,
				_Animations(HashMap::from([(
					AnimationPriority::High,
					OrderedSet::from([AnimationKey::Open, AnimationKey::Idle]),
				)])),
			));

			entity.remove::<IsActive>();

			assert_eq!(
				Some(&_Animations(HashMap::from([(
					AnimationPriority::High,
					OrderedSet::from([AnimationKey::Close])
				)]))),
				entity.get::<_Animations>(),
			);
		}

		#[test]
		fn do_nothing_when_door_missing() {
			let mut app = setup();
			let mut entity = app.world_mut().spawn((IsActive, _Animations::default()));

			entity.remove::<IsActive>();

			assert_eq!(Some(&_Animations::default()), entity.get::<_Animations>(),);
		}
	}
}
