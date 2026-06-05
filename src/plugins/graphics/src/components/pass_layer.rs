use bevy::{camera::visibility::Layer, prelude::*};
use std::collections::{HashSet, hash_set::Iter as HashSetIter};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct PassLayers {
	layer: Layer,
	additional_layers: HashSet<Layer>,
}

impl PassLayers {
	pub(crate) fn add_layer(&mut self, layer: Layer) {
		if self.layer == layer {
			return;
		}

		self.additional_layers.insert(layer);
	}

	pub(crate) fn reset(&mut self) {
		self.additional_layers.clear();
	}

	pub(crate) fn iter(&self) -> Iter<'_> {
		self.into_iter()
	}

	pub(crate) fn contains(&self, layer: &Layer) -> bool {
		&self.layer == layer || self.additional_layers.contains(layer)
	}
}

impl From<Layer> for PassLayers {
	fn from(layer: Layer) -> Self {
		Self {
			layer,
			additional_layers: HashSet::default(),
		}
	}
}

impl<'a> IntoIterator for &'a PassLayers {
	type Item = Layer;
	type IntoIter = Iter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		Iter {
			layers: self,
			additional_layers: None,
		}
	}
}

pub(crate) struct Iter<'a> {
	layers: &'a PassLayers,
	additional_layers: Option<HashSetIter<'a, Layer>>,
}

impl Iterator for Iter<'_> {
	type Item = Layer;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.additional_layers {
			None => {
				self.additional_layers = Some(self.layers.additional_layers.iter());
				Some(self.layers.layer)
			}
			Some(it) => it.next().copied(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::camera::visibility::RenderLayers;
	use testing::assert_count;

	#[test]
	fn single_layer() {
		let pass_layer = PassLayers::from(42);

		assert_eq!(
			RenderLayers::layer(42),
			RenderLayers::from_iter(&pass_layer)
		);
	}

	#[test]
	fn multiple_layers() {
		let mut pass_layer = PassLayers::from(42);

		pass_layer.add_layer(11);
		pass_layer.add_layer(22);

		assert_eq!(
			RenderLayers::from_layers(&[42, 11, 22]),
			RenderLayers::from_iter(&pass_layer),
		);
	}

	#[test]
	fn do_not_base_layer() {
		let mut pass_layer = PassLayers::from(42);

		pass_layer.add_layer(42);

		assert_count!(1, pass_layer.iter());
	}

	#[test]
	fn do_not_repeat_added_layers() {
		let mut pass_layer = PassLayers::from(42);

		pass_layer.add_layer(11);
		pass_layer.add_layer(11);

		assert_count!(2, pass_layer.iter());
	}

	#[test]
	fn reset() {
		let mut pass_layer = PassLayers::from(42);

		pass_layer.add_layer(11);
		pass_layer.add_layer(22);
		pass_layer.reset();

		assert_eq!(
			RenderLayers::layer(42),
			RenderLayers::from_iter(&pass_layer),
		);
	}

	#[test]
	fn contains_layer() {
		let pass_layer = PassLayers::from(42);

		assert!(pass_layer.contains(&42));
	}

	#[test]
	fn does_not_contain_layer() {
		let pass_layer = PassLayers::from(42);

		assert!(!pass_layer.contains(&11));
	}

	#[test]
	fn contains_additional_layer() {
		let mut pass_layer = PassLayers::from(42);

		pass_layer.add_layer(11);

		assert!(pass_layer.contains(&11));
	}

	#[test]
	fn does_not_contain_additional_layer() {
		let mut pass_layer = PassLayers::from(42);

		pass_layer.add_layer(11);

		assert!(!pass_layer.contains(&22));
	}
}
