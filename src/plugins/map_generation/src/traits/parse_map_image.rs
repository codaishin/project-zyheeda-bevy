pub(crate) trait ParseMapImage<TImage>: Sized {
	type TParseError;
	type TLookup;

	fn try_parse(image: &TImage, lookup: &Self::TLookup) -> Result<Self, Self::TParseError>;
}
