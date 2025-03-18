use bevy::math::Dir3;
use common::traits::load_asset::Path;
use std::vec::IntoIter;

#[derive(Debug, PartialEq, Default, Clone)]
pub(crate) struct ModelData(Vec<(Path, Dir3)>);

impl<TPath> FromIterator<(TPath, Dir3)> for ModelData
where
	Path: From<TPath>,
{
	fn from_iter<TValues>(values: TValues) -> Self
	where
		TValues: IntoIterator<Item = (TPath, Dir3)>,
	{
		Self(
			values
				.into_iter()
				.map(|(path, dir)| (Path::from(path), dir))
				.collect(),
		)
	}
}

impl IntoIterator for ModelData {
	type Item = (Path, Dir3);
	type IntoIter = IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}
