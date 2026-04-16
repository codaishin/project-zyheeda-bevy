use crate::components::GlobalZIndexTop;
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

pub(crate) fn adjust_global_z_index(
	mut commands: ZyheedaCommands,
	global_z_tops: Query<(Entity, Ref<GlobalZIndexTop>)>,
) {
	let top_z_index = global_z_tops.iter().filter(is_not_new).count() as i32;
	let new_indices = (top_z_index + 1)..;
	let new_global_z_tops = global_z_tops.iter().filter(is_new);

	for (index, (entity, ..)) in new_indices.zip(new_global_z_tops) {
		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(GlobalZIndex(index));
		});
	}
}

fn is_new((.., global_z_index_top): &(Entity, Ref<GlobalZIndexTop>)) -> bool {
	global_z_index_top.is_added()
}

fn is_not_new((.., global_z_index_top): &(Entity, Ref<GlobalZIndexTop>)) -> bool {
	!global_z_index_top.is_added()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::app::{App, Update};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, adjust_global_z_index);

		app
	}

	#[test]
	fn set_z_index_to_one() {
		let mut app = setup();

		let entity = app.world_mut().spawn(GlobalZIndexTop).id();

		app.update();

		let entity = app.world().entity(entity);

		assert_eq!(Some(&GlobalZIndex(1)), entity.get::<GlobalZIndex>())
	}

	#[test]
	fn set_z_index_to_count_of_entities() {
		let mut app = setup();

		let entities = [
			app.world_mut().spawn(GlobalZIndexTop).id(),
			app.world_mut().spawn(GlobalZIndexTop).id(),
		];

		app.update();

		let entities = entities.map(|entity| app.world().entity(entity));

		assert_eq!(
			[Some(&GlobalZIndex(1)), Some(&GlobalZIndex(2))],
			entities.map(|entity| entity.get::<GlobalZIndex>())
		)
	}

	#[test]
	fn do_not_set_z_index_when_component_missing() {
		let mut app = setup();

		let entities = [
			app.world_mut().spawn(GlobalZIndexTop).id(),
			app.world_mut().spawn_empty().id(),
		];

		app.update();

		let entities = entities.map(|entity| app.world().entity(entity));

		assert_eq!(
			[Some(&GlobalZIndex(1)), None],
			entities.map(|entity| entity.get::<GlobalZIndex>())
		)
	}

	#[test]
	fn set_z_index_to_count_of_entities_incrementally_to_previous_update() {
		let mut app = setup();

		let before_update = app.world_mut().spawn(GlobalZIndexTop).id();

		app.update();

		let entities = [
			before_update,
			app.world_mut().spawn(GlobalZIndexTop).id(),
			app.world_mut().spawn(GlobalZIndexTop).id(),
		];

		app.update();

		let entities = entities.map(|entity| app.world().entity(entity));

		assert_eq!(
			[
				Some(&GlobalZIndex(1)),
				Some(&GlobalZIndex(2)),
				Some(&GlobalZIndex(3))
			],
			entities.map(|entity| entity.get::<GlobalZIndex>())
		)
	}
}
