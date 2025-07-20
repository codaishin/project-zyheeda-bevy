use bevy::prelude::*;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct AgentsColorLookup {
	pub(crate) player: Color,
	pub(crate) enemy: Color,
}

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct AgentsColorLookupImages<TImage = Image>
where
	TImage: Asset,
{
	pub(crate) player: Handle<TImage>,
	pub(crate) enemy: Handle<TImage>,
}
