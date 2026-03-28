use crate::traits::accessors::get::{View, ViewField};
use std::ops::Deref;

impl<T, F> View<F> for T
where
	F: ViewField,
	T: Deref<Target: View<F>>,
{
	fn view(&self) -> <F as ViewField>::TValue<'_> {
		self.deref().view()
	}
}
