use crate::{
	components::animation_dispatch::AnimationDispatch,
	system_params::animations::AnimationsContextMut,
};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::TryApplyOn,
		animation::{AnimationKey, AnimationPriority, OverrideAnimations},
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::HashSet;

impl<TServer> OverrideAnimations for AnimationsContextMut<'_, TServer> {
	fn override_animations<TLayer, TAnimations>(&mut self, layer: TLayer, animations: TAnimations)
	where
		TLayer: Into<AnimationPriority> + 'static,
		TAnimations: IntoIterator<Item = AnimationKey> + 'static,
	{
		let entity = self.entity.id();
		self.entity.trigger_observers_for(AnimationOverrideEvent {
			entity,
			priority: layer.into(),
			animations: animations.into_iter().collect(),
		});
	}
}

#[derive(Event)]
pub(crate) struct AnimationOverrideEvent {
	entity: Entity,
	priority: AnimationPriority,
	animations: HashSet<AnimationKey>,
}

impl AnimationOverrideEvent {
	pub(crate) fn observe(
		trigger: Trigger<Self>,
		mut commands: ZyheedaCommands,
		mut dispatchers: Query<&mut AnimationDispatch<AnimationKey>>,
	) {
		let Self {
			entity,
			priority,
			animations,
		} = trigger.event();

		if let Ok(mut dispatch) = dispatchers.get_mut(*entity) {
			dispatch.override_animations(*priority, animations.clone());
			return;
		}

		commands.try_apply_on(entity, |mut e| {
			let mut dispatch = AnimationDispatch::default();
			dispatch.override_animations(*priority, animations.clone());
			e.try_insert(dispatch);
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::animation_dispatch::AnimationDispatch,
		system_params::animations::AnimationsParamMut,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{accessors::get::GetContextMut, animation::Animations},
	};
	use testing::SingleThreadedApp;

	#[derive(Resource)]
	struct _Server;

	fn dispatch_with<TOverrides, TLayer, TKeys>(
		overrides: TOverrides,
	) -> AnimationDispatch<AnimationKey>
	where
		TOverrides: IntoIterator<Item = (TLayer, TKeys)>,
		TLayer: Into<AnimationPriority> + 'static,
		TKeys: IntoIterator<Item = AnimationKey> + 'static,
	{
		let mut dispatch = AnimationDispatch::default();
		for (layer, keys) in overrides {
			dispatch.override_animations(layer, keys);
		}
		dispatch
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(AnimationOverrideEvent::observe);
		app.insert_resource(_Server);
		app.insert_resource(Assets::<AnimationGraph>::default());

		app
	}

	#[test]
	fn apply_animations() -> Result<(), RunSystemError> {
		let mut app = setup();
		let animations = [AnimationKey::Run, AnimationKey::Skill(SlotKey(11))];
		let entity = app
			.world_mut()
			.spawn(AnimationDispatch::<AnimationKey>::default())
			.id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server>| {
				let key = Animations { entity };
				let mut ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();
				ctx.override_animations(AnimationPriority::Low, animations);
			})?;

		assert_eq!(
			Some(&dispatch_with([(AnimationPriority::Low, animations)])),
			app.world()
				.entity(entity)
				.get::<AnimationDispatch<AnimationKey>>(),
		);
		Ok(())
	}

	#[test]
	fn insert_new_dispatch_if_not_already_present() -> Result<(), RunSystemError> {
		let mut app = setup();
		let animations = [AnimationKey::Run, AnimationKey::Skill(SlotKey(11))];
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server>| {
				let key = Animations { entity };
				let mut ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();
				ctx.override_animations(AnimationPriority::Low, animations);
			})?;

		assert_eq!(
			Some(&dispatch_with([(AnimationPriority::Low, animations)])),
			app.world()
				.entity(entity)
				.get::<AnimationDispatch<AnimationKey>>(),
		);
		Ok(())
	}

	#[test]
	fn set_multiple_overrides_when_initially_not_present() -> Result<(), RunSystemError> {
		let mut app = setup();
		let animations = [AnimationKey::Run, AnimationKey::Skill(SlotKey(11))];
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server>| {
				let key = Animations { entity };
				let mut ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();
				ctx.override_animations(AnimationPriority::Low, animations);
				ctx.override_animations(AnimationPriority::High, animations);
			})?;

		assert_eq!(
			Some(&dispatch_with([
				(AnimationPriority::Low, animations),
				(AnimationPriority::High, animations)
			])),
			app.world()
				.entity(entity)
				.get::<AnimationDispatch<AnimationKey>>(),
		);
		Ok(())
	}
}
