use super::track_in_self_and_children;
use crate::traits::track::{IsTracking, Track, Untrack};
use bevy::prelude::*;

type UntrackFunc<TTracker, TTarget> = fn(RemovedComponents<TTarget>, Query<Mut<TTracker>>);

type TrackFunc<TTracker, TTarget, TFilter> = fn(
	Query<(Entity, Mut<TTracker>)>,
	Query<&TTarget, (Added<TTarget>, With<TFilter>)>,
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
	TFilter: Component,
{
	pub(super) track: TrackFunc<TTracker, TTarget, TFilter>,
	pub(super) untrack: UntrackFunc<TTracker, TTarget>,
}

impl<TTracker, TTarget, TFilter> TrackSystemsBuilder<TTracker, TTarget, TFilter>
where
	TTracker: Component,
	TTarget: Component,
	TFilter: Component,
{
	pub fn system(self) -> TrackSystems<TTracker, TTarget, TFilter> {
		(self.track, self.untrack)
	}
}

impl<TTracker, TTarget> TrackSystemsBuilder<TTracker, TTarget, TTarget>
where
	TTracker: Component + Track<TTarget> + Untrack<TTarget> + IsTracking<TTarget>,
	TTarget: Component,
{
	pub fn with<TFilter>(self) -> TrackSystemsBuilder<TTracker, TTarget, TFilter>
	where
		TFilter: Component,
	{
		TrackSystemsBuilder {
			track: track_in_self_and_children::<TTracker, TTarget, TFilter>,
			untrack: self.untrack,
		}
	}
}
