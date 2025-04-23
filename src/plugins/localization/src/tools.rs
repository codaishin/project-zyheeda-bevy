use std::fmt::Display;

pub(crate) fn list_string<T>(items: &[T]) -> String
where
	T: Display,
{
	items
		.iter()
		.map(|e| format!("  - {e}"))
		.collect::<Vec<_>>()
		.join("\n")
}
