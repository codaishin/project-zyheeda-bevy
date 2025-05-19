use super::track_in_self_and_children;
use crate::traits::track::{IsTracking, Track, Untrack};
use bevy::{
	ecs::{component::Mutable, query::QueryFilter},
	prelude::*,
};

type UntrackFunc<TTracker, TTarget> = fn(RemovedComponents<TTarget>, Query<Mut<TTracker>>);

type TrackFunc<TTracker, TTarget, TFilter> = fn(
	Query<(Entity, Mut<TTracker>)>,
	Query<&TTarget, (Added<TTarget>, TFilter)>,
	Query<(), With<TTracker>>,
	Query<&Children>,
);

type TrackSystems<TTracker, TTarget, TFilter> = (
	TrackFunc<TTracker, TTarget, TFilter>,
	UntrackFunc<TTracker, TTarget>,
);

pub struct TrackSystemsBuilder<TTracker, TTarget, TFilter>
where
	TTracker: Component,
	TTarget: Component,
	TFilter: QueryFilter,
{
	pub(super) track: TrackFunc<TTracker, TTarget, TFilter>,
	pub(super) untrack: UntrackFunc<TTracker, TTarget>,
}

impl<TTracker, TTarget, TFilter> TrackSystemsBuilder<TTracker, TTarget, TFilter>
where
	TTracker: Component,
	TTarget: Component,
	TFilter: QueryFilter,
{
	pub fn system(self) -> TrackSystems<TTracker, TTarget, TFilter> {
		(self.track, self.untrack)
	}
}

impl<TTracker, TTarget> TrackSystemsBuilder<TTracker, TTarget, ()>
where
	TTracker:
		Component<Mutability = Mutable> + Track<TTarget> + Untrack<TTarget> + IsTracking<TTarget>,
	TTarget: Component,
{
	pub fn filter<TFilter>(self) -> TrackSystemsBuilder<TTracker, TTarget, TFilter>
	where
		TFilter: QueryFilter,
	{
		TrackSystemsBuilder {
			track: track_in_self_and_children::<TTracker, TTarget, TFilter>,
			untrack: self.untrack,
		}
	}
}
