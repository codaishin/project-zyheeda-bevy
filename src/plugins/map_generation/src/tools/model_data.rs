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

#[cfg(test)]
pub(crate) mod test_tools {
	#[derive(Debug, PartialEq)]
	pub(crate) struct Compare {
		pub(crate) len: usize,
		pub(crate) match_left: bool,
		pub(crate) match_right: bool,
	}

	#[macro_export]
	macro_rules! assert_eq_model_data {
		($left:expr, $right:expr) => {{
			let left_vec = $left.into_iter().collect::<Vec<_>>();
			let right_vec = $right.into_iter().collect::<Vec<_>>();

			assert_eq!(
				$crate::tools::model_data::test_tools::Compare {
					len: left_vec.len(),
					match_left: true,
					match_right: left_vec.iter().all(|v| right_vec.contains(v)),
				},
				$crate::tools::model_data::test_tools::Compare {
					len: right_vec.len(),
					match_left: right_vec.iter().all(|v| left_vec.contains(v)),
					match_right: true,
				},
				"\n  left_data: {:?}\n right_data: {:?}",
				$left,
				$right
			)
		}};
	}
}
