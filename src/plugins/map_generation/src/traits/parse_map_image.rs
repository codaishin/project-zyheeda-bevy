use crate::resources::map::color_lookup::MapColorLookup;

pub(crate) trait ParseMapImage<TImage, TCell>: Sized {
	type TParseError;

	fn try_parse(image: &TImage, lookup: &MapColorLookup<TCell>) -> Result<Self, Self::TParseError>;
}
