use crate::components::{interactive::Interactive, interactive_state::IsActive};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	traits::{
		accessors::get::{ContextChanged, GetContext, TryApplyOn},
		handles_physics::{InteractionsOngoing, IterInteractions},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl Interactive {
	pub(crate) fn reset_when_no_interactions<TInteractions>(
		mut commands: ZyheedaCommands,
		interactive_entities: Query<Entity, With<Self>>,
		interactions: StaticSystemParam<TInteractions>,
	) where
		TInteractions:
			SystemParam + for<'c> GetContext<InteractionsOngoing, TContext<'c>: IterInteractions>,
	{
		for entity in interactive_entities {
			let ctx = TInteractions::get_context(&interactions, InteractionsOngoing { entity });

			if !ctx.context_changed() {
				continue;
			}

			if ctx.iter_interactions().len() > 0 {
				continue;
			}

			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<IsActive>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::interactive_state::IsActive;
	use common::traits::handles_map_generation::InteractiveType;
	use std::{iter::Copied, ops::DerefMut, slice::Iter};
	use testing::{SingleThreadedApp, fake_entity};

	#[derive(Resource, Debug, PartialEq)]
	struct _Interactions(Vec<Entity>);

	impl _Interactions {
		fn from_entities(entities: impl Into<Vec<Entity>>) -> Self {
			Self(entities.into())
		}
	}

	impl IterInteractions for _Interactions {
		type TIter<'a>
			= Copied<Iter<'a, Entity>>
		where
			Self: 'a;

		fn iter_interactions(&self) -> Self::TIter<'_> {
			self.0.iter().copied()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(_Interactions::from_entities([]));
		app.add_systems(
			Update,
			Interactive::reset_when_no_interactions::<Res<_Interactions>>,
		);

		app
	}

	#[test]
	fn remove_active_when_no_interactions_present() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Interactive {
					interactive_type: InteractiveType::Door,
				},
				IsActive,
			))
			.id();

		app.update();

		assert!(!app.world().entity(entity).contains::<IsActive>());
	}

	#[test]
	fn do_not_remove_active_when_interactions_present() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Interactive {
					interactive_type: InteractiveType::Door,
				},
				IsActive,
			))
			.id();
		app.insert_resource(_Interactions::from_entities([fake_entity!(11)]));

		app.update();

		assert!(app.world().entity(entity).contains::<IsActive>());
	}

	#[test]
	fn do_nothing_if_interactive_missing() {
		let mut app = setup();
		let entity = app.world_mut().spawn(IsActive).id();

		app.update();

		assert!(app.world().entity(entity).contains::<IsActive>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Interactive {
					interactive_type: InteractiveType::Door,
				},
				IsActive,
			))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).insert(IsActive);
		app.update();

		assert!(app.world().entity(entity).contains::<IsActive>());
	}

	#[test]
	fn act_again_if_interactions_change() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Interactive {
					interactive_type: InteractiveType::Door,
				},
				IsActive,
			))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).insert(IsActive);
		app.world_mut().resource_mut::<_Interactions>().deref_mut();
		app.update();

		assert!(!app.world().entity(entity).contains::<IsActive>());
	}
}
