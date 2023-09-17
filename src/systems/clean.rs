use bevy::prelude::{Component, Query};

use crate::traits::clean::Clean;

pub fn clean<TCleanable: Clean + Component>(mut query: Query<&mut TCleanable>) {
	for mut elem in query.iter_mut() {
		elem.clean();
	}
}

#[cfg(test)]
mod tests {
	use bevy::prelude::{App, Update};
	use mockall::automock;

	use super::*;

	#[derive(Component)]
	struct _Cleanable {
		pub mock: Mock_Cleanable,
	}

	impl _Cleanable {
		fn new() -> Self {
			Self {
				mock: Mock_Cleanable::new(),
			}
		}
	}

	#[automock]
	impl Clean for _Cleanable {
		fn clean(&mut self) {
			self.mock.clean()
		}
	}

	#[test]
	fn call_clean() {
		let mut app = App::new();
		let mut cleanable = _Cleanable::new();

		cleanable.mock.expect_clean().times(1).return_const(());

		app.world.spawn(cleanable);
		app.add_systems(Update, clean::<_Cleanable>);

		app.update();
	}
}
