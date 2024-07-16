use crate::traits::Spawn;
use bevy::{
	ecs::system::{Commands, In},
	transform::components::Transform,
};

pub(crate) fn spawn_procedural<TCell: Spawn>(
	cells: In<Vec<(Transform, TCell)>>,
	mut commands: Commands,
) {
	for (transform, cell) in cells.0 {
		cell.spawn(&mut commands, transform);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::{component::Component, system::IntoSystem},
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Clone)]
	struct _Cell;

	#[derive(Component, Debug, PartialEq)]
	struct _Spawned;

	impl Spawn for _Cell {
		fn spawn(&self, commands: &mut Commands, at: Transform) {
			commands.spawn((_Spawned, at));
		}
	}

	fn setup(cells: Vec<(Transform, _Cell)>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			(move || cells.clone()).pipe(spawn_procedural::<_Cell>),
		);

		app
	}

	#[test]
	fn spawn() {
		let mut app = setup(vec![(Transform::from_xyz(1., 2., 3.), _Cell)]);
		app.update();

		let spawned = app
			.world()
			.iter_entities()
			.find_map(|e| Some((e.get::<_Spawned>()?, e.get::<Transform>()?)));

		assert_eq!(Some((&_Spawned, &Transform::from_xyz(1., 2., 3.))), spawned);
	}
}
