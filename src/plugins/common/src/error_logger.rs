use crate::errors::ErrorData;
use bevy::{ecs::system::SystemParam, prelude::*};
use std::{
	any::TypeId,
	collections::{HashMap, hash_map::Entry},
	ops::Deref,
	sync::{LazyLock, Mutex},
	time::Instant,
};

pub trait Log {
	fn log<TError>(&self, error: TError)
	where
		TError: ErrorData;
}

impl<T> Log for T
where
	T: Deref<Target: Log>,
{
	fn log<TError>(&self, error: TError)
	where
		TError: ErrorData,
	{
		self.deref().log(error);
	}
}

static CACHE: LazyLock<ErrorLoggerCache> = LazyLock::new(ErrorLoggerCache::default);

#[derive(SystemParam)]
pub struct ErrorLogger {
	_p: (),
}

impl ErrorLogger {
	/// A global logger. This works, because [`ErrorLogger`] is a fake system parameter.
	/// However, for testability it is recommended to test against generic system parameters
	/// implementing [`Log`] and then injecting the [`ErrorLogger`] type as dependency.
	pub const GLOBAL: Self = Self { _p: () };
}

impl Log for ErrorLogger {
	fn log<TError>(&self, error: TError)
	where
		TError: ErrorData,
	{
		if CACHE.is_suppressed::<TError>() {
			return;
		}

		Self::write(error);
	}
}

#[cfg(not(test))]
mod prod {
	use super::*;
	use crate::errors::Level;
	use tracing::{error, field::display, warn};

	impl ErrorLogger {
		pub(super) fn write<TError>(error: TError)
		where
			TError: ErrorData,
		{
			let level = error.level();
			let label = TError::label();
			let details = display(error.into_details());

			match level {
				Level::Error => {
					error!(details, "{label}");
				}
				Level::Warning => {
					warn!(details, "{label}");
				}
			}
		}
	}
}

#[cfg(test)]
mod test {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::errors::Level;
	use std::sync::Mutex;

	pub(super) static LOG: Mutex<Vec<LogEntry>> = Mutex::new(vec![]);

	pub(super) fn clear_cache() {
		let mut log = LOG.lock().unwrap();

		log.clear();

		let mut cache = CACHE.0.lock().unwrap();

		cache.clear();
	}

	#[derive(Debug, PartialEq, Clone)]
	pub(super) struct LogEntry {
		pub(super) level: Level,
		pub(super) label: String,
		pub(super) details: String,
	}

	impl LogEntry {
		pub(super) fn get_all() -> Vec<LogEntry> {
			let log = LOG.lock().unwrap();

			log.to_vec()
		}
	}

	impl ErrorLogger {
		pub(super) fn write<TError>(error: TError)
		where
			TError: ErrorData,
		{
			let level = error.level();
			let label = TError::label().to_string();
			let details = error.into_details().to_string();

			let mut log = LOG.lock().unwrap();

			log.push(LogEntry {
				level,
				label,
				details,
			});
		}
	}
}

#[derive(Debug, Default)]
pub(crate) struct ErrorLoggerCache(Mutex<HashMap<TypeId, Instant>>);

impl ErrorLoggerCache {
	fn is_suppressed<TError>(&self) -> bool
	where
		TError: ErrorData + 'static,
	{
		let Some(limit) = TError::rate_limit() else {
			return false;
		};

		// SAFETY: This only fails if another user panicked while holding the mutex.
		#[allow(clippy::expect_used)]
		let mut cache = self
			.0
			.lock()
			.expect("Cannot obtain error cache: mutex poisoned");

		match cache.entry(TypeId::of::<TError>()) {
			Entry::Occupied(e) if e.get().elapsed() < limit => return true,
			Entry::Occupied(mut e) => {
				e.insert(Instant::now());
			}
			Entry::Vacant(e) => {
				e.insert(Instant::now());
			}
		}

		false
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{error_logger::test::LogEntry, errors::Level};
	use std::{sync::Mutex, thread, time::Duration};

	static LOCK: Mutex<()> = Mutex::new(());

	struct _Error;

	impl ErrorData for _Error {
		fn level(&self) -> Level {
			Level::Error
		}

		fn label() -> impl std::fmt::Display {
			"ERROR"
		}

		fn into_details(self) -> impl std::fmt::Display {
			"DETAILS"
		}
	}

	struct _LimitedError;

	impl ErrorData for _LimitedError {
		fn rate_limit() -> Option<Duration> {
			Some(Duration::from_millis(500))
		}

		fn level(&self) -> Level {
			Level::Error
		}

		fn label() -> impl std::fmt::Display {
			"LIMITED ERROR"
		}

		fn into_details(self) -> impl std::fmt::Display {
			"DETAILS"
		}
	}

	macro_rules! locked {
		($test:expr) => {{
			let _lock = LOCK.lock().unwrap();
			test::clear_cache();
			$test;
		}};
	}

	#[test]
	fn log_error() {
		locked! {{
			ErrorLogger::GLOBAL.log(_Error);

			assert_eq!(
				vec![LogEntry {
					level: Level::Error,
					label: String::from("ERROR"),
					details: String::from("DETAILS")
				}],
				LogEntry::get_all(),
			);
		}}
	}

	#[test]
	fn limit_rate() {
		locked! {{
			ErrorLogger::GLOBAL.log(_LimitedError);
			ErrorLogger::GLOBAL.log(_LimitedError);

			assert_eq!(
				vec![LogEntry {
					level: Level::Error,
					label: String::from("LIMITED ERROR"),
					details: String::from("DETAILS")
				}],
				LogEntry::get_all(),
			);
		}}
	}

	#[test]
	fn log_again_after_rate_limit_expired() {
		locked! {{
			ErrorLogger::GLOBAL.log(_LimitedError);
			thread::sleep(Duration::from_millis(750));
			ErrorLogger::GLOBAL.log(_LimitedError);

			assert_eq!(
				vec![
					LogEntry {
						level: Level::Error,
						label: String::from("LIMITED ERROR"),
						details: String::from("DETAILS")
					},
					LogEntry {
						level: Level::Error,
						label: String::from("LIMITED ERROR"),
						details: String::from("DETAILS")
					},
				],
				LogEntry::get_all(),
			);
		}}
	}

	#[test]
	fn log_again_when_no_rate_limit() {
		locked! {{
			ErrorLogger::GLOBAL.log(_Error);
			ErrorLogger::GLOBAL.log(_Error);

			assert_eq!(
				vec![
					LogEntry {
						level: Level::Error,
						label: String::from("ERROR"),
						details: String::from("DETAILS")
					},
					LogEntry {
						level: Level::Error,
						label: String::from("ERROR"),
						details: String::from("DETAILS")
					},
				],
				LogEntry::get_all(),
			);
		}}
	}
}
