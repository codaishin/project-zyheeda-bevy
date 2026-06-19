use bevy::prelude::*;
use std::ops::{Add, Div, Mul, Sub};

#[derive(Default, Debug, PartialEq)]
pub struct Pixel(pub f32);

impl From<Pixel> for Val {
	fn from(value: Pixel) -> Self {
		Val::Px(value.0)
	}
}

impl From<Pixel> for UiRect {
	fn from(value: Pixel) -> Self {
		UiRect::all(Val::from(value))
	}
}

impl Sub for Pixel {
	type Output = Pixel;

	fn sub(self, rhs: Self) -> Self::Output {
		Pixel(self.0 - rhs.0)
	}
}

impl Add for Pixel {
	type Output = Pixel;

	fn add(self, rhs: Self) -> Self::Output {
		Pixel(self.0 + rhs.0)
	}
}

impl Div<f32> for Pixel {
	type Output = Pixel;

	fn div(self, rhs: f32) -> Self::Output {
		Pixel(self.0 / rhs)
	}
}

impl Mul<f32> for Pixel {
	type Output = Pixel;

	fn mul(self, rhs: f32) -> Self::Output {
		Pixel(self.0 * rhs)
	}
}
