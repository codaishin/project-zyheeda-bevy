pub mod components;
mod systems;
mod traits;

use bevy::{
	app::{App, Plugin, Update},
	camera::{Camera, visibility::RenderLayers},
	ecs::schedule::IntoScheduleConfigs,
};
use common::{
	attributes::health::Health,
	components::ui_node_for::UiNodeFor,
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
		let update_life_bars =
			bar::<TPhysics::TAffectedComponent, Health, Camera, TGraphics::TUiCamera>;
		let render_life_bars = render_bar::<Health>;
		let render_layer = UiNodeFor::<Bar>::render_layer::<TGraphics::TUiCamera>;

		app.register_required_components::<TAgents::TAgent, Bar>();
		app.register_required_components_with::<UiNodeFor<Bar>, RenderLayers>(render_layer);

		app.manage_ownership::<Bar>(Update);
		app.add_systems(Update, (update_life_bars, render_life_bars).chain());
	}
}
