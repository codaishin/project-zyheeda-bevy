use crate::{
	components::map::cells::parsed_color::ParsedColor,
	grid_graph::grid_context::CellDistance,
	resources::agents::color_lookup::AgentsColorLookup,
	traits::{
		GridCellDistanceDefinition,
		map_cells_extra::MapCellsExtra,
		parse_map_image::ParseMapImage,
	},
};
use common::errors::Unreachable;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(crate) enum Agent<TCell> {
	#[default]
	None,
	Player,
	Enemy,
	_C(PhantomData<TCell>, Unreachable),
}

impl<TCell> GridCellDistanceDefinition for Agent<TCell>
where
	TCell: GridCellDistanceDefinition,
{
	const CELL_DISTANCE: CellDistance = TCell::CELL_DISTANCE;
}

impl<TCell> ParseMapImage<ParsedColor> for Agent<TCell> {
	type TParseError = Unreachable;
	type TLookup = AgentsColorLookup;

	fn try_parse(
		color: &ParsedColor,
		lookup: &AgentsColorLookup,
	) -> Result<Self, Self::TParseError> {
		match color.color().as_ref() {
			Some(color) if color == &&lookup.player => Ok(Self::Player),
			Some(color) if color == &&lookup.enemy => Ok(Self::Enemy),
			_ => Ok(Self::None),
		}
	}
}

impl<TCell> MapCellsExtra for Agent<TCell> {
	type TExtra = ();
}
