use crate::resources::save_context::SaveContext;
use bevy::prelude::*;
use serde::Serialize;

#[derive(Component, Debug, PartialEq, Default)]
pub struct Save {
	fns: Vec<fn(&mut SaveContext, &EntityRef)>,
}

impl Save {
	pub fn handling<TComponent>(mut self) -> Self
	where
		TComponent: Component + Serialize + 'static,
	{
		self.fns.push(SaveContext::save::<TComponent>);

		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Component, Serialize)]
	struct _A;

	#[derive(Component, Serialize)]
	struct _B;

	#[test]
	fn store_save_fn() {
		let save = Save::default();

		let save = save.handling::<_A>();

		assert_eq!(
			Save {
				fns: vec![SaveContext::save::<_A>]
			},
			save
		);
	}

	#[test]
	fn store_save_fns() {
		let save = Save::default();

		let save = save.handling::<_A>().handling::<_B>();

		assert_eq!(
			Save {
				fns: vec![SaveContext::save::<_A>, SaveContext::save::<_B>]
			},
			save
		);
	}
}
