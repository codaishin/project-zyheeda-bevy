use crate::traits::Flush;
use bevy::{
	ecs::component::{Component, Mutable},
	prelude::Query,
};

pub(crate) fn flush<TFlush>(mut agents: Query<&mut TFlush>)
where
	TFlush: Flush + Component<Mutability = Mutable>,
{
	for mut queue in &mut agents {
		queue.flush();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::{App, Update};
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::automock;

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
		app.add_systems(Update, flush::<_Dequeue>);

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
