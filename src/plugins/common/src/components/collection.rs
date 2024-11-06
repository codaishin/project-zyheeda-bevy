use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub struct Collection<TElement>(pub Vec<TElement>);

impl<TElement> Collection<TElement> {
	pub fn new<const N: usize>(elements: [TElement; N]) -> Self {
		Self(elements.into())
	}
}
