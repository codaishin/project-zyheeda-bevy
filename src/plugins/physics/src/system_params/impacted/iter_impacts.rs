use crate::system_params::impacted::ImpactedContext;
use common::prelude::*;

impl IterImpacts for ImpactedContext {
	type TIter<'a>
		= std::iter::Empty<Impact>
	where
		Self: 'a;

	fn iter_impacts(&self) -> Self::TIter<'_> {
		std::iter::empty()
	}
}
