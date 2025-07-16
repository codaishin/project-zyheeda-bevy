use crate::resources::color_lookup::ColorLookup;

pub(crate) trait ParseMapImage<TImage, TCell>: Sized {
	type TParseError;

	fn try_parse(image: &TImage, lookup: &ColorLookup<TCell>) -> Result<Self, Self::TParseError>;
}
