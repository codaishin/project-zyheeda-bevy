use bevy::ecs::{component::Component, system::Query};

use crate::traits::{FlushObsolete, Priority};

pub(crate) fn flush<TAnimationDispatch: Component + FlushObsolete>(
	mut dispatches: Query<&mut TAnimationDispatch>,
) {
	for mut dispatch in &mut dispatches {
		dispatch.flush_obsolete(Priority::High);
		dispatch.flush_obsolete(Priority::Middle);
		dispatch.flush_obsolete(Priority::Low);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::Priority;
	use bevy::app::{App, Update};
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component, NestedMocks)]
	struct _Dispatch {
		mock: Mock_Dispatch,
	}

	#[automock]
	impl FlushObsolete for _Dispatch {
		fn flush_obsolete(&mut self, priority: crate::traits::Priority) {
			self.mock.flush_obsolete(priority)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, flush::<_Dispatch>);

		app
	}

	#[test]
	fn call_flush() {
		let mut app = setup();
		app.world_mut().spawn(_Dispatch::new().with_mock(|mock| {
			mock.expect_flush_obsolete()
				.times(1)
				.with(eq(Priority::High))
				.return_const(());
			mock.expect_flush_obsolete()
				.times(1)
				.with(eq(Priority::Middle))
				.return_const(());
			mock.expect_flush_obsolete()
				.times(1)
				.with(eq(Priority::Low))
				.return_const(());
		}));

		app.update();
	}
}
