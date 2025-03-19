use crate::{grid_graph::GridGraph, traits::to_subdivided::ToSubdivided};
use bevy::prelude::*;
use common::traits::{thread_safe::ThreadSafe, try_insert_on::TryInsertOn};

#[derive(Component, Debug, PartialEq)]
#[require(Name(Self::name), Transform, Visibility)]
pub struct Level<const SUBDIVISIONS: u8 = 0, TGraph = GridGraph>
where
	TGraph: ToSubdivided,
{
	graph: TGraph,
}

impl<const SUBDIVISIONS: u8, TGraph> Level<SUBDIVISIONS, TGraph>
where
	TGraph: ToSubdivided,
{
	fn name() -> String {
		format!("Level (subdivisions: {SUBDIVISIONS})")
	}

	pub(crate) fn insert(
		mut commands: Commands,
		levels: Query<(Entity, &Level<0, TGraph>), Changed<Level<0, TGraph>>>,
	) where
		TGraph: ThreadSafe,
	{
		for (entity, level) in &levels {
			let graph = level.graph.to_subdivided(SUBDIVISIONS);
			commands.try_insert_on(entity, Self { graph });
		}
	}
}

impl Default for Level {
	fn default() -> Self {
		Self {
			graph: Default::default(),
		}
	}
}

impl From<GridGraph> for Level {
	fn from(graph: GridGraph) -> Self {
		Level { graph }
	}
}

impl<const SUBDIVISIONS: u8> From<&Level<SUBDIVISIONS>> for GridGraph {
	fn from(value: &Level<SUBDIVISIONS>) -> Self {
		value.graph.clone()
	}
}

#[cfg(test)]
mod test_insert_subdivided {
	use super::*;

	#[derive(Debug, PartialEq)]
	struct _Graph {
		subdivisions: u8,
	}

	impl ToSubdivided for _Graph {
		fn to_subdivided(&self, subdivisions: u8) -> Self {
			_Graph { subdivisions }
		}
	}

	fn setup<const SUBDIVISIONS: u8>() -> App {
		let mut app = App::new();
		app.add_systems(Update, Level::<SUBDIVISIONS, _Graph>::insert);

		app
	}

	#[test]
	fn spawn_subdivided() {
		let mut app = setup::<5>();
		let entity = app
			.world_mut()
			.spawn(Level::<0, _Graph> {
				graph: _Graph { subdivisions: 0 },
			})
			.id();

		app.update();

		assert_eq!(
			Some(&Level {
				graph: _Graph { subdivisions: 5 }
			}),
			app.world().entity(entity).get::<Level::<5, _Graph>>()
		);
	}

	#[test]
	fn do_not_insert_when_level_not_changed() {
		let mut app = setup::<5>();
		let entity = app
			.world_mut()
			.spawn(Level::<0, _Graph> {
				graph: _Graph { subdivisions: 0 },
			})
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<Level<5, _Graph>>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<Level::<5, _Graph>>());
	}

	#[test]
	fn insert_again_when_level_changed() {
		let mut app = setup::<5>();
		let entity = app
			.world_mut()
			.spawn(Level::<0, _Graph> {
				graph: _Graph { subdivisions: 0 },
			})
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<Level<5, _Graph>>()
			.get_mut::<Level<0, _Graph>>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&Level {
				graph: _Graph { subdivisions: 5 }
			}),
			app.world().entity(entity).get::<Level::<5, _Graph>>()
		);
	}
}
