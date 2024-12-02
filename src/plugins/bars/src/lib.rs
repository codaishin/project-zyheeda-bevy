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
		handles_life::HandlesLife,
		ownership_relation::OwnershipRelation,
	},
};
use components::Bar;
use std::marker::PhantomData;
use systems::{bar::bar, render_bar::render_bar};

pub struct BarsPlugin<TLifeCyclePlugin>(PhantomData<TLifeCyclePlugin>);

impl<TLifeCyclePlugin> BarsPlugin<TLifeCyclePlugin>
where
	TLifeCyclePlugin: Plugin + HandlesLife,
{
	pub fn depends_on(_: &TLifeCyclePlugin) -> Self {
		Self(PhantomData)
	}
}

impl<TLifeCyclePlugin> Plugin for BarsPlugin<TLifeCyclePlugin>
where
	TLifeCyclePlugin: Plugin + HandlesLife,
{
	fn build(&self, app: &mut App) {
		let get_health = TLifeCyclePlugin::TLife::get;
		let update_life_bars = bar::<TLifeCyclePlugin::TLife, Health, Camera>(get_health);
		let render_life_bars = render_bar::<Health>;

		app.manage_ownership::<Bar>(Update);
		app.add_systems(Update, (update_life_bars, render_life_bars).chain());
	}
}
