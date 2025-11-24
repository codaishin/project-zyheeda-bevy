use crate::components::{animation_lookup::AnimationLookup, setup_animations::SetupAnimations};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl SetupAnimations {
	pub(crate) fn stop(
		mut commands: ZyheedaCommands,
		setups: Query<Entity, (With<AnimationLookup>, With<Self>)>,
	) {
		for entity in &setups {
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<Self>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, SetupAnimations::stop);

		app
	}

	#[test]
	fn remove_setup_animations() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((AnimationLookup::default(), SetupAnimations))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SetupAnimations>());
	}

	#[test]
	fn do_not_remove_setup_animations_when_no_animation_lookup_present() {
		let mut app = setup();
		let entity = app.world_mut().spawn(SetupAnimations).id();

		app.update();

		assert_eq!(
			Some(&SetupAnimations),
			app.world().entity(entity).get::<SetupAnimations>(),
		);
	}
}
