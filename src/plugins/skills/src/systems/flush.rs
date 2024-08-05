use crate::traits::Flush;
use bevy::{ecs::component::Component, prelude::Query};

pub(crate) fn flush<TFlush: Flush + Component>(mut agents: Query<&mut TFlush>) {
	for mut queue in &mut agents {
		queue.flush();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::{App, Update};
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMock};
	use macros::NestedMock;
	use mockall::automock;

	#[derive(Component, NestedMock)]
	struct _Dequeue {
		mock: Mock_Dequeue,
	}

	#[automock]
	impl Flush for _Dequeue {
		fn flush(&mut self) {
			self.mock.flush()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, flush::<_Dequeue>);

		app
	}

	#[test]
	fn call_flush() {
		let mut app = setup();
		app.world_mut().spawn(_Dequeue::new_mock(|mock| {
			mock.expect_flush().times(1).return_const(());
		}));

		app.update();
	}
}
