use crate::traits::process_delta::ProcessDelta;
use bevy::prelude::*;

pub fn asset_process_delta<TAsset, TTime>(
	time: Res<Time<TTime>>,
	mut assets: ResMut<Assets<TAsset>>,
) where
	TAsset: Asset + ProcessDelta,
	TTime: Default + Sync + Send + 'static,
{
	let delta = time.delta();

	for (_, asset) in assets.iter_mut() {
		asset.process_delta(delta);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::TickTime;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::time::Duration;

	#[derive(Asset, TypePath, NestedMocks)]
	struct _Asset {
		mock: Mock_Asset,
	}

	#[automock]
	impl ProcessDelta for _Asset {
		fn process_delta(&mut self, delta: Duration) {
			self.mock.process_delta(delta);
		}
	}

	fn setup() -> App {
		let mut app = App::new();
		app.init_resource::<Time<Real>>();
		app.init_resource::<Assets<_Asset>>();
		app.tick_time(Duration::ZERO);

		app
	}

	#[test]
	fn call_with_time_delta() -> Result<(), RunSystemError> {
		let mut app = setup();
		let asset = _Asset::new().with_mock(assert);
		app.world_mut().resource_mut::<Assets<_Asset>>().add(asset);

		app.tick_time(Duration::from_millis(42));
		app.world_mut()
			.run_system_once(asset_process_delta::<_Asset, Real>)?;

		fn assert(mock: &mut Mock_Asset) {
			mock.expect_process_delta()
				.times(1)
				.with(eq(Duration::from_millis(42)))
				.return_const(());
		}
		Ok(())
	}
}
