use bevy::{camera::visibility::Layer, prelude::*};
use std::collections::{
	HashSet,
	hash_set::{IntoIter as HashSetIntoIter, Iter as HashSetIter},
};

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct ModelRenderLayers {
	main: MainLayers,
	additional: HashSet<Layer>,
}

impl ModelRenderLayers {
	pub(crate) fn add_layers<T>(&mut self, layers: T)
	where
		T: IntoIterator<Item = Layer>,
	{
		let not_main_layer = layers
			.into_iter()
			.filter(|layer| !self.main.contains(layer));

		self.additional.extend(not_main_layer);
	}

	pub(crate) fn reset(&mut self) {
		self.additional.clear();
	}

	pub(crate) fn iter(&self) -> Iter<'_> {
		self.into_iter()
	}

	pub(crate) fn contains_all<'a, T>(&'a self, layers: T) -> bool
	where
		T: IntoIterator<Item = &'a Layer>,
	{
		let miss = |layer| !self.main.contains(layer) && !self.additional.contains(layer);

		!layers.into_iter().any(miss)
	}
}

impl From<Layer> for ModelRenderLayers {
	fn from(layer: Layer) -> Self {
		Self {
			main: MainLayers::Single(layer),
			additional: HashSet::default(),
		}
	}
}

impl From<&'static [Layer]> for ModelRenderLayers {
	fn from(layers: &'static [Layer]) -> Self {
		Self {
			main: MainLayers::Multiple(layers),
			additional: HashSet::default(),
		}
	}
}

impl<'a> IntoIterator for &'a ModelRenderLayers {
	type Item = &'a Layer;
	type IntoIter = Iter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		Iter {
			main: self.main.iter(),
			additional: self.additional.iter(),
		}
	}
}

impl IntoIterator for ModelRenderLayers {
	type Item = Layer;
	type IntoIter = IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			main: self.main.into_iter(),
			additional: self.additional.into_iter(),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum MainLayers {
	Single(Layer),
	Multiple(&'static [Layer]),
}

impl MainLayers {
	fn contains(&self, layer: &Layer) -> bool {
		match self {
			MainLayers::Single(single) => single == layer,
			MainLayers::Multiple(multiple) => multiple.contains(layer),
		}
	}

	fn iter(&self) -> MainLayersIter<'_> {
		match self {
			MainLayers::Single(layer) => MainLayersIter::Single(Some(layer)),
			MainLayers::Multiple(layers) => MainLayersIter::Multiple(layers.iter()),
		}
	}

	fn into_iter(self) -> MainLayersIntoIter {
		match self {
			MainLayers::Single(layer) => MainLayersIntoIter::Single(Some(layer)),
			MainLayers::Multiple(layers) => MainLayersIntoIter::Multiple(layers.iter()),
		}
	}
}

enum MainLayersIter<'a> {
	Single(Option<&'a Layer>),
	Multiple(std::slice::Iter<'a, Layer>),
}

pub(crate) struct Iter<'a> {
	main: MainLayersIter<'a>,
	additional: HashSetIter<'a, Layer>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = &'a Layer;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.main {
			MainLayersIter::Single(None) => self.additional.next(),
			MainLayersIter::Single(main) => main.take(),
			MainLayersIter::Multiple(main) => main.next().or_else(|| self.additional.next()),
		}
	}
}

enum MainLayersIntoIter {
	Single(Option<Layer>),
	Multiple(std::slice::Iter<'static, Layer>),
}

pub(crate) struct IntoIter {
	main: MainLayersIntoIter,
	additional: HashSetIntoIter<Layer>,
}

impl Iterator for IntoIter {
	type Item = Layer;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.main {
			MainLayersIntoIter::Single(None) => self.additional.next(),
			MainLayersIntoIter::Single(main) => main.take(),
			MainLayersIntoIter::Multiple(main) => {
				main.next().copied().or_else(|| self.additional.next())
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::assert_count;

	#[test]
	fn single_layer() {
		let pass_layer = ModelRenderLayers::from(42);

		assert_eq!(
			HashSet::from([42]),
			pass_layer.into_iter().collect::<HashSet<_>>(),
		);
	}

	#[test]
	fn multiple_layers() {
		let mut pass_layer = ModelRenderLayers::from(42);

		pass_layer.add_layers([11]);
		pass_layer.add_layers([22]);

		assert_eq!(
			HashSet::from([42, 11, 22]),
			pass_layer.into_iter().collect::<HashSet<_>>(),
		);
	}

	#[test]
	fn do_not_repeat_base_layer() {
		let mut pass_layer = ModelRenderLayers::from(42);

		pass_layer.add_layers([42]);

		assert_count!(1, pass_layer.iter());
	}

	#[test]
	fn do_not_repeat_base_layers() {
		const MAIN: &[Layer] = &[42, 333];
		let mut pass_layer = ModelRenderLayers::from(MAIN);

		pass_layer.add_layers([42, 333]);

		assert_count!(2, pass_layer.iter());
	}

	#[test]
	fn do_not_repeat_added_layers() {
		let mut pass_layer = ModelRenderLayers::from(42);

		pass_layer.add_layers([11]);
		pass_layer.add_layers([11]);

		assert_count!(2, pass_layer.iter());
	}

	#[test]
	fn reset() {
		let mut pass_layer = ModelRenderLayers::from(42);

		pass_layer.add_layers([11]);
		pass_layer.add_layers([22]);
		pass_layer.reset();

		assert_eq!(
			HashSet::from([42]),
			pass_layer.into_iter().collect::<HashSet<_>>(),
		);
	}

	#[test]
	fn into_iter_ref() {
		let mut pass_layer = ModelRenderLayers::from(42);

		pass_layer.add_layers([11]);
		pass_layer.add_layers([22]);

		assert_eq!(
			HashSet::from([42, 11, 22]),
			(&pass_layer).into_iter().copied().collect::<HashSet<_>>(),
		);
	}

	#[test]
	fn into_iter_of_multiple_main_layers() {
		const MAIN: &[Layer] = &[42, 333];
		let mut pass_layer = ModelRenderLayers::from(MAIN);

		pass_layer.add_layers([11]);
		pass_layer.add_layers([22]);

		assert_eq!(
			HashSet::from([42, 333, 11, 22]),
			pass_layer.into_iter().collect::<HashSet<_>>(),
		);
	}

	#[test]
	fn iter_of_multiple_main_layers() {
		const MAIN: &[Layer] = &[42, 333];
		let mut pass_layer = ModelRenderLayers::from(MAIN);

		pass_layer.add_layers([11]);
		pass_layer.add_layers([22]);

		assert_eq!(
			HashSet::from([&42, &333, &11, &22]),
			pass_layer.iter().collect::<HashSet<_>>(),
		);
	}

	#[test]
	fn contains_layer() {
		let pass_layer = ModelRenderLayers::from(42);

		assert!(pass_layer.contains_all(&[42]));
	}

	#[test]
	fn does_not_contain_layer() {
		let pass_layer = ModelRenderLayers::from(42);

		assert!(!pass_layer.contains_all(&[11]));
	}

	#[test]
	fn contains_additional_layer() {
		let mut pass_layer = ModelRenderLayers::from(42);

		pass_layer.add_layers([11]);

		assert!(pass_layer.contains_all(&[11]));
	}

	#[test]
	fn does_not_contain_additional_layer() {
		let mut pass_layer = ModelRenderLayers::from(42);

		pass_layer.add_layers([11]);

		assert!(!pass_layer.contains_all(&[22]));
	}

	#[test]
	fn contains_mixed() {
		let mut pass_layer = ModelRenderLayers::from(42);
		pass_layer.add_layers([11]);

		assert!(pass_layer.contains_all(&[42, 11]));
	}

	#[test]
	fn does_not_contain_mixed_main_miss() {
		let mut pass_layer = ModelRenderLayers::from(42);
		pass_layer.add_layers([11]);

		assert!(!pass_layer.contains_all(&[3, 11]));
	}

	#[test]
	fn does_not_contain_mixed_additional_miss() {
		let mut pass_layer = ModelRenderLayers::from(42);
		pass_layer.add_layers([11]);

		assert!(!pass_layer.contains_all(&[42, 3]));
	}
}
