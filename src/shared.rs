// std
use std::{
	sync::atomic::{AtomicU16, Ordering},
	thread::{self, JoinHandle},
};
// crates.io
use once_cell::sync::{Lazy, OnceCell};
// cargo-featalign
use crate::cli::{IndentSymbol, Mode, SharedInitiator};

pub static FEATURES: OnceCell<Vec<String>> = OnceCell::new();
pub static MODE: OnceCell<Mode> = OnceCell::new();
pub static INDENTATION: OnceCell<String> = OnceCell::new();

static THREAD: OnceCell<u16> = OnceCell::new();
static THREAD_ACTIVE: Lazy<AtomicU16> = Lazy::new(|| AtomicU16::new(1));
// TODO: optimize to `active_threads`
pub fn activate_thread<F, T>(threads_pool: &mut Vec<JoinHandle<T>>, f: F)
where
	F: 'static + Send + FnOnce() -> T,
	T: 'static + Send,
{
	if THREAD_ACTIVE.load(Ordering::SeqCst) < *THREAD.get().unwrap() - 1 {
		THREAD_ACTIVE.fetch_add(1, Ordering::SeqCst);

		threads_pool.push(thread::spawn(f));
	} else {
		f();
	}
}
pub fn deactivate_threads<T>(threads: Vec<JoinHandle<T>>) -> Vec<T> {
	let ts_count = threads.len() as u16;
	let rs = threads.into_iter().map(|t| t.join().unwrap()).collect();

	THREAD_ACTIVE.fetch_sub(ts_count, Ordering::SeqCst);

	rs
}

pub struct Shared;
impl Shared {
	pub fn initialize(initiator: SharedInitiator) -> Self {
		FEATURES.set(initiator.features).unwrap();
		THREAD.set(initiator.thread).unwrap();
		MODE.set(initiator.mode).unwrap();

		let indentation = match initiator.indent_symbol {
			IndentSymbol::Tab => "\n\t".into(),
			IndentSymbol::Whitespace => format!("\n{}", " ".repeat(initiator.indent_size)),
		};

		INDENTATION.set(indentation).unwrap();

		Self
	}
}
