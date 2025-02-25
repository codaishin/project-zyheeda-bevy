use bevy::prelude::*;
use common::traits::{handles_map_generation::NavCell, iterate::Iterate};

#[derive(Component, Debug, PartialEq, Default)]
pub struct NavGrid(Vec<NavCell>);

impl NavGrid {
	pub(crate) fn update_from<TMap>(mut maps: Query<(&TMap, &mut Self), Changed<TMap>>)
	where
		for<'a> TMap: Component + Iterate<TItem<'a> = &'a NavCell>,
	{
		for (map, mut nav_grid) in &mut maps {
			*nav_grid = Self(map.iterate().copied().collect());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Map(Vec<NavCell>);

	impl Iterate for _Map {
		type TItem<'a>
			= &'a NavCell
		where
			Self: 'a;

		fn iterate(&self) -> impl Iterator<Item = &'_ NavCell> {
			self.0.iter()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, NavGrid::update_from::<_Map>);

		app
	}

	#[test]
	fn do_update() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				NavGrid::default(),
				_Map(vec![NavCell {
					translation: Vec3::new(1., 2., 3.),
					..default()
				}]),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&NavGrid(vec![NavCell {
				translation: Vec3::new(1., 2., 3.),
				..default()
			}])),
			app.world().entity(entity).get::<NavGrid>(),
		);
	}

	#[test]
	fn no_update_if_map_unchanged() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				NavGrid::default(),
				_Map(vec![NavCell {
					translation: Vec3::new(1., 2., 3.),
					..default()
				}]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(NavGrid::default());
		app.update();

		assert_eq!(
			Some(&NavGrid(vec![])),
			app.world().entity(entity).get::<NavGrid>(),
		);
	}

	#[test]
	fn update_again_if_map_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				NavGrid::default(),
				_Map(vec![NavCell {
					translation: Vec3::new(1., 2., 3.),
					..default()
				}]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(NavGrid::default())
			.get_mut::<_Map>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&NavGrid(vec![NavCell {
				translation: Vec3::new(1., 2., 3.),
				..default()
			}])),
			app.world().entity(entity).get::<NavGrid>(),
		);
	}
}
