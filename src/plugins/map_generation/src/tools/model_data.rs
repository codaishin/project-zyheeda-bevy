use bevy::math::Dir3;
use common::traits::load_asset::Path;
use std::vec::IntoIter;

#[derive(Debug, PartialEq, Default, Clone)]
pub(crate) struct ModelData(Vec<(Path, Dir3)>);

impl<const N: usize, TPath> From<[(TPath, Dir3); N]> for ModelData
where
	Path: From<TPath>,
{
	fn from(value: [(TPath, Dir3); N]) -> Self {
		Self(
			value
				.map(|(path, dir)| (Path::from(path), dir))
				.into_iter()
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
