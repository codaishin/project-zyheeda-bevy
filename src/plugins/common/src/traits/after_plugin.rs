use crate::tools::plugin_system_set::PluginSystemSet;
use bevy::{
	ecs::schedule::{Chain, GraphInfo, Schedulable, ScheduleConfigs},
	prelude::*,
};

pub trait AfterPlugin<T, Marker>: IntoScheduleConfigs<T, Marker>
where
	T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>,
{
	fn after_plugin<S>(self, PluginSystemSet(set): PluginSystemSet<S>) -> ScheduleConfigs<T>
	where
		S: SystemSet,
	{
		self.after(set)
	}
}

impl<S, T, Marker> AfterPlugin<T, Marker> for S
where
	S: IntoScheduleConfigs<T, Marker>,
	T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>,
{
}
