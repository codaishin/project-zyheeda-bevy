use common::traits::load_asset::Path;
use std::vec::IntoIter;

#[derive(Debug, PartialEq, Default, Clone)]
pub(crate) struct Paths(Vec<Path>);

impl<const N: usize, TPath> From<[TPath; N]> for Paths
where
	Path: From<TPath>,
{
	fn from(value: [TPath; N]) -> Self {
		Self(value.map(Path::from).into_iter().collect())
	}
}

impl IntoIterator for Paths {
	type Item = Path;
	type IntoIter = IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}
