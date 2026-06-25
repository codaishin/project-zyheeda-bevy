pub mod components;
mod systems;
mod traits;

use bevy::{
	app::{App, Plugin, Update},
	ecs::schedule::IntoScheduleConfigs,
};
use common::{
	attributes::health::Health,
	traits::{
		handles_agents::HandlesAgents,
		handles_graphics::UiCamera,
		handles_physics::HandlesLife,
		ownership_relation::OwnershipRelation,
		thread_safe::ThreadSafe,
	},
};
use components::bar::Bar;
use std::marker::PhantomData;
use systems::{bar::bar, render_bar::render_bar};

pub struct BarsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TAgents, TPhysics, TGraphics> BarsPlugin<(TAgents, TPhysics, TGraphics)>
where
	TAgents: ThreadSafe + HandlesAgents,
	TPhysics: ThreadSafe + HandlesLife,
	TGraphics: ThreadSafe + UiCamera,
{
	pub fn from_plugins(_: &TAgents, _: &TPhysics, _: &TGraphics) -> Self {
		Self(PhantomData)
	}
}

impl<TAgents, TPhysics, TGraphics> Plugin for BarsPlugin<(TAgents, TPhysics, TGraphics)>
where
	TAgents: ThreadSafe + HandlesAgents,
	TPhysics: ThreadSafe + HandlesLife,
	TGraphics: ThreadSafe + UiCamera,
{
	fn build(&self, app: &mut App) {
		let update_life_bars = bar::<TPhysics::TAffectedComponent, Health, TGraphics::TUiCameraMut>;
		let render_life_bars = render_bar::<Health, TGraphics::TUiCameraMut>;

		app.register_required_components::<TAgents::TAgent, Bar>();

		app.manage_ownership::<Bar>(Update);
		app.add_systems(Update, (update_life_bars, render_life_bars).chain());
	}
}
