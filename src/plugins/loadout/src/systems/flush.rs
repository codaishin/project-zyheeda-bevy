use crate::traits::Flush;
use bevy::{
	ecs::component::{Component, Mutable},
	prelude::Query,
};

impl<T> FlushSystem for T where T: Flush + Component<Mutability = Mutable> {}

pub(crate) trait FlushSystem: Flush + Component<Mutability = Mutable> + Sized {
	fn flush_system(mut agents: Query<&mut Self>) {
		for mut queue in &mut agents {
			queue.flush();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::{App, Update};
	use macros::NestedMocks;
	use mockall::automock;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks)]
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
		app.add_systems(Update, _Dequeue::flush_system);

		app
	}

	#[test]
	fn call_flush() {
		let mut app = setup();
		app.world_mut().spawn(_Dequeue::new().with_mock(|mock| {
			mock.expect_flush().times(1).return_const(());
		}));

		app.update();
	}
}
