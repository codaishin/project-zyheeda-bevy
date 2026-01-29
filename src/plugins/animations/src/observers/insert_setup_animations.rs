use crate::components::setup_animations::SetupAnimations;
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl SetupAnimations {
	pub(crate) fn insert_when<TEvent>(trigger: On<TEvent>, mut commands: ZyheedaCommands)
	where
		TEvent: EntityEvent,
	{
		commands.try_apply_on(&trigger.event_target(), |mut e| {
			e.try_insert(Self);
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	#[derive(EntityEvent)]
	struct _Event(Entity);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(SetupAnimations::insert_when::<_Event>);

		app
	}

	#[test]
	fn insert_setup_animations() {
		let mut app = setup();
		let mut entity = app.world_mut().spawn_empty();

		let entity = entity.trigger(_Event).id();
		app.update();

		assert_eq!(
			Some(&SetupAnimations),
			app.world().entity(entity).get::<SetupAnimations>(),
		);
	}
}
