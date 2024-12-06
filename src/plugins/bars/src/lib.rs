pub mod components;
mod systems;
mod traits;

use bevy::{
	app::{App, Plugin, Update},
	prelude::IntoSystemConfigs,
	render::camera::Camera,
};
use common::{
	attributes::health::Health,
	traits::{
		accessors::get::GetterRef,
		handles_bars::HandlesBars,
		handles_life::HandlesLife,
		ownership_relation::OwnershipRelation,
	},
};
use components::Bar;
use std::marker::PhantomData;
use systems::{bar::bar, render_bar::render_bar};

pub struct BarsPlugin<TLifeCycle>(PhantomData<TLifeCycle>);

impl<TLifeCycle> BarsPlugin<TLifeCycle> {
	pub fn depends_on(_: &TLifeCycle) -> Self {
		Self(PhantomData)
	}
}

impl<TLifeCycle> Plugin for BarsPlugin<TLifeCycle>
where
	TLifeCycle: Plugin + HandlesLife,
{
	fn build(&self, app: &mut App) {
		let get_health = TLifeCycle::TLife::get;
		let update_life_bars = bar::<TLifeCycle::TLife, Health, Camera>(get_health);
		let render_life_bars = render_bar::<Health>;

		app.manage_ownership::<Bar>(Update);
		app.add_systems(Update, (update_life_bars, render_life_bars).chain());
	}
}

impl<TLifeCycle> HandlesBars for BarsPlugin<TLifeCycle> {
	type TBar = Bar;

	fn new_bar() -> Self::TBar {
		Bar::default()
	}
}
