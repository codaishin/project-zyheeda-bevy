use bevy::{
	ecs::query::{QueryData, QueryFilter},
	prelude::Query,
};
use std::marker::PhantomData;

pub(crate) struct Either<TFn, D1, F1>
where
	TFn: FnMut(Query<D1, F1>) -> bool,
{
	phantom_data: PhantomData<(D1, F1)>,
	fn1: TFn,
}

impl<TFn1, TD1, TF1> Either<TFn1, TD1, TF1>
where
	TFn1: FnMut(Query<TD1, TF1>) -> bool,
	TD1: QueryData,
	TF1: QueryFilter,
{
	pub(crate) fn or<TFn2, TD2, TF2>(
		self,
		mut fn2: TFn2,
	) -> impl FnMut(Query<TD1, TF1>, Query<TD2, TF2>) -> bool
	where
		TFn2: FnMut(Query<TD2, TF2>) -> bool + 'static,
		TD2: QueryData,
		TF2: QueryFilter,
	{
		let mut fn1 = self.fn1;
		move |query1, query2| fn1(query1) || fn2(query2)
	}
}

pub(crate) fn either<TFn1, D1, F1>(fn1: TFn1) -> Either<TFn1, D1, F1>
where
	TFn1: FnMut(Query<D1, F1>) -> bool + 'static,
	D1: QueryData,
	F1: QueryFilter,
{
	Either {
		fn1,
		phantom_data: PhantomData,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::App,
		ecs::system::RunSystemOnce,
		prelude::{Component, IntoSystem},
	};

	#[derive(Component)]
	struct _C1;

	#[derive(Component)]
	struct _C2;

	fn run_system<T: IntoSystem<(), Out, Marker>, Out, Marker>(system: T) -> Out {
		App::new().world_mut().run_system_once(system)
	}

	#[test]
	fn true_when_both_true() {
		fn f1(_: Query<&_C1>) -> bool {
			true
		}
		fn f2(_: Query<&_C2>) -> bool {
			true
		}

		assert!(run_system(either(f1).or(f2)));
	}

	#[test]
	fn false_when_both_false() {
		fn f1(_: Query<&_C1>) -> bool {
			false
		}
		fn f2(_: Query<&_C2>) -> bool {
			false
		}

		assert!(!run_system(either(f1).or(f2)));
	}

	#[test]
	fn true_when_only_first_true() {
		fn f1(_: Query<&_C1>) -> bool {
			true
		}
		fn f2(_: Query<&_C2>) -> bool {
			false
		}

		assert!(run_system(either(f1).or(f2)));
	}

	#[test]
	fn true_when_only_second_true() {
		fn f1(_: Query<&_C1>) -> bool {
			false
		}
		fn f2(_: Query<&_C2>) -> bool {
			true
		}

		assert!(run_system(either(f1).or(f2)));
	}
}
