use bevy::{camera::visibility::Layer, prelude::*};
use std::collections::{
	HashSet,
	hash_set::{IntoIter as HashSetIntoIter, Iter as HashSetIter},
};

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct ModelRenderLayers {
	main: Layer,
	additional: HashSet<Layer>,
}

impl ModelRenderLayers {
	pub(crate) fn add_layers<T>(&mut self, layers: T)
	where
		T: IntoIterator<Item = Layer>,
	{
		let not_main_layer = layers.into_iter().filter(|layer| layer != &self.main);

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
		let miss = |layer| &self.main != layer && !self.additional.contains(layer);

		!layers.into_iter().any(miss)
	}
}

impl From<Layer> for ModelRenderLayers {
	fn from(layer: Layer) -> Self {
		Self {
			main: layer,
			additional: HashSet::default(),
		}
	}
}

impl<'a> IntoIterator for &'a ModelRenderLayers {
	type Item = &'a Layer;
	type IntoIter = Iter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		Iter {
			main: Some(&self.main),
			additional: self.additional.iter(),
		}
	}
}

impl IntoIterator for ModelRenderLayers {
	type Item = Layer;
	type IntoIter = IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			main: Some(self.main),
			additional: self.additional.into_iter(),
		}
	}
}

pub(crate) struct Iter<'a> {
	main: Option<&'a Layer>,
	additional: HashSetIter<'a, Layer>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = &'a Layer;

	fn next(&mut self) -> Option<Self::Item> {
		match self.main.take() {
			Some(layer) => Some(layer),
			None => self.additional.next(),
		}
	}
}

pub(crate) struct IntoIter {
	main: Option<Layer>,
	additional: HashSetIntoIter<Layer>,
}

impl Iterator for IntoIter {
	type Item = Layer;

	fn next(&mut self) -> Option<Self::Item> {
		match self.main.take() {
			Some(layer) => Some(layer),
			None => self.additional.next(),
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
	fn do_not_base_layer() {
		let mut pass_layer = ModelRenderLayers::from(42);

		pass_layer.add_layers([42]);

		assert_count!(1, pass_layer.iter());
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
