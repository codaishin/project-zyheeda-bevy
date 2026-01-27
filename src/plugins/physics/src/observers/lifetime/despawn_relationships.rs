use crate::components::lifetime::TiedLifetimes;
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl TiedLifetimes {
	pub(crate) fn despawn_relationships_on_remove(
		trigger: Trigger<OnRemove, Self>,
		tied_lifetimes: Query<&Self>,
		mut commands: ZyheedaCommands,
	) {
		let Ok(tied_lifetimes) = tied_lifetimes.get(trigger.target()) else {
			return;
		};

		for entity in tied_lifetimes.iter() {
			commands.try_apply_on(&entity, |e| e.try_despawn());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Component)]
	struct _Related;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(TiedLifetimes::despawn_relationships_on_remove);

		app
	}

	#[test]
	fn despawn_relationships_on_despawn_target() {
		let mut app = setup();
		let target = app.world_mut().spawn(related!(TiedLifetimes[
			_Related,
			_Related,
			_Related,
		]));

		target.despawn();

		assert_count!(
			0,
			app.world()
				.iter_entities()
				.filter(|e| e.contains::<_Related>())
		);
	}
}
