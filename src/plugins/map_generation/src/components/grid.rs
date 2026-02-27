use crate::{
	grid_graph::GridGraph,
	traits::to_subdivided::{SubdivisionError, ToSubdivided},
};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaCommands,
};

#[derive(Component, Debug, PartialEq)]
#[require(Name = Self::name(), Transform, Visibility)]
#[component(immutable)]
pub struct Grid<const SUBDIVISIONS: u8 = 0, TGraph = GridGraph>
where
	TGraph: ToSubdivided,
{
	graph: TGraph,
}

impl<const SUBDIVISIONS: u8, TGraph> Grid<SUBDIVISIONS, TGraph>
where
	TGraph: ToSubdivided,
{
	fn name() -> String {
		format!("Grid (subdivisions: {SUBDIVISIONS})")
	}

	pub(crate) fn insert(
		mut commands: ZyheedaCommands,
		levels: Query<(Entity, &Grid<0, TGraph>), Changed<Grid<0, TGraph>>>,
	) -> Result<(), Vec<SubdivisionError>>
	where
		TGraph: ThreadSafe,
	{
		let errors = levels
			.iter()
			.filter_map(
				|(entity, level)| match level.graph.to_subdivided(SUBDIVISIONS) {
					Ok(graph) => {
						commands.try_apply_on(&entity, |mut e| {
							e.try_insert(Self { graph });
						});
						None
					}
					Err(err) => Some(err),
				},
			)
			.collect::<Vec<_>>();

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}
}

impl Default for Grid {
	fn default() -> Self {
		Self {
			graph: Default::default(),
		}
	}
}

impl From<&GridGraph> for Grid {
	fn from(graph: &GridGraph) -> Self {
		Grid {
			graph: graph.clone(),
		}
	}
}

impl<const SUBDIVISIONS: u8> From<&Grid<SUBDIVISIONS>> for GridGraph {
	fn from(value: &Grid<SUBDIVISIONS>) -> Self {
		value.graph.clone()
	}
}
