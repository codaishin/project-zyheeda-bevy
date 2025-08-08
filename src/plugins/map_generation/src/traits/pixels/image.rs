use crate::traits::pixels::{Bytes, PixelBytes};
use bevy::prelude::*;

impl PixelBytes for Image {
	fn pixel_bytes(&self, coords: UVec3) -> Option<Bytes<'_>> {
		self.pixel_bytes(coords)
	}
}
