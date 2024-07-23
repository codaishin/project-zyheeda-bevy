use crate::components::tooltip::TooltipUI;
use bevy::{ecs::system::Res, prelude::Query, render::view::Visibility, time::Time};
use std::time::Duration;

pub(crate) fn tooltip_visibility<
	TTime: Send + Sync + 'static + Default,
	T: Send + Sync + 'static,
>(
	mut uis: Query<(&mut TooltipUI<T>, &mut Visibility)>,
	time: Res<Time<TTime>>,
) {
	let delta = time.delta();

	for (mut ui, mut visibility) in &mut uis {
		if delta < ui.delay {
			ui.delay -= delta;
		} else {
			ui.delay = Duration::ZERO;
			*visibility = Visibility::Visible;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::Entity,
		time::Real,
	};
	use common::test_tools::utils::{SingleThreadedApp, TickTime};
	use std::time::Duration;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Time<Real>>();
		app.tick_time(Duration::ZERO);
		app.add_systems(Update, tooltip_visibility::<Real, ()>);

		app
	}

	#[test]
	fn set_to_visible_after_delay() {
		let mut app = setup();
		let tooltip_ui = app
			.world_mut()
			.spawn((
				TooltipUI::<()>::new(Entity::from_raw(42), Duration::from_millis(100)),
				Visibility::Hidden,
			))
			.id();

		app.tick_time(Duration::from_millis(100));
		app.update();

		let tooltip_ui = app.world().entity(tooltip_ui);

		assert_eq!(Some(&Visibility::Visible), tooltip_ui.get::<Visibility>());
	}

	#[test]
	fn do_not_set_to_visible_before_delay() {
		let mut app = setup();
		let tooltip_ui = app
			.world_mut()
			.spawn((
				TooltipUI::<()>::new(Entity::from_raw(42), Duration::from_millis(10)),
				Visibility::Hidden,
			))
			.id();

		app.tick_time(Duration::from_millis(9));
		app.update();

		let tooltip_ui = app.world().entity(tooltip_ui);

		assert_eq!(Some(&Visibility::Hidden), tooltip_ui.get::<Visibility>());
	}

	#[test]
	fn set_to_visible_after_delay_reached_in_successive_updates() {
		let mut app = setup();
		let tooltip_ui = app
			.world_mut()
			.spawn((
				TooltipUI::<()>::new(Entity::from_raw(42), Duration::from_millis(1000)),
				Visibility::Hidden,
			))
			.id();

		app.tick_time(Duration::from_millis(500));
		app.update();

		app.tick_time(Duration::from_millis(500));
		app.update();

		let tooltip_ui = app.world().entity(tooltip_ui);

		assert_eq!(Some(&Visibility::Visible), tooltip_ui.get::<Visibility>());
	}

	#[test]
	fn set_delay_to_zero_when_delta_exceeds_delay() {
		let mut app = setup();
		let tooltip_ui = app
			.world_mut()
			.spawn((
				TooltipUI::<()>::new(Entity::from_raw(42), Duration::from_millis(10)),
				Visibility::Hidden,
			))
			.id();

		app.tick_time(Duration::from_millis(11));
		app.update();

		let tooltip_ui = app.world().entity(tooltip_ui);

		assert_eq!(
			Some(&TooltipUI::<()>::new(Entity::from_raw(42), Duration::ZERO)),
			tooltip_ui.get::<TooltipUI<()>>()
		);
	}
}
