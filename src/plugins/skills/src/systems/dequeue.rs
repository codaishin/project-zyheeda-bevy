use crate::{
	skill::{Queued, Skill},
	traits::TryDequeue,
};
use bevy::{ecs::component::Component, prelude::Query};

pub(crate) fn dequeue<TDequeue: TryDequeue<Skill<Queued>> + Component>(
	mut agents: Query<&mut TDequeue>,
) {
	for mut queue in &mut agents {
		queue.try_dequeue();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::skill::{Queued, Skill};
	use bevy::prelude::{App, Update};
	use common::test_tools::utils::SingleThreadedApp;
	use mockall::automock;

	#[derive(Component, Default)]
	struct _Dequeue {
		mock: Mock_Dequeue,
	}

	#[automock]
	impl TryDequeue<Skill<Queued>> for _Dequeue {
		fn try_dequeue(&mut self) {
			self.mock.try_dequeue()
		}
	}

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, dequeue::<_Dequeue>);

		app
	}

	#[test]
	fn try_dequeue() {
		let mut app = setup();
		let mut queue = _Dequeue::default();
		queue.mock.expect_try_dequeue().times(1).return_const(());

		app.world.spawn(queue);

		app.update();
	}
}
