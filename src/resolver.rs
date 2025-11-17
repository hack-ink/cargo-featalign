// std
use std::{
	fs::{self, File},
	io::{BufWriter, Write},
	mem,
};
// crates.io
use cargo_metadata::PackageId;
use fxhash::FxHashMap;
use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use toml_edit::{DocumentMut, Value, visit_mut::VisitMut};
// cargo-featalign
use crate::{
	analyzer::{PROBLEMS, Problem, ProblemCrate},
	cli::{Mode, ResolverInitiator},
	prelude::*,
	shared::{FEATURES, INDENTATION, MODE},
	sorter::SortVisitor,
};

static PATH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"path\+file://(/.+?)#").unwrap());

static SORT: OnceCell<bool> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct Resolver;
impl Resolver {
	pub fn initialize(initiator: ResolverInitiator) -> Self {
		SORT.set(initiator.sort).unwrap();

		Self
	}

	pub fn resolve(self) -> Result<()> {
		if *MODE.get().unwrap() == Mode::Check {
			return Ok(());
		}

		let ps = mem::take(&mut *PROBLEMS.lock().unwrap());
		let mut ts = Vec::new();

		ps.into_iter().for_each(|(c, pcs)| {
			let r = self.clone();

			shared::activate_thread(&mut ts, move || r.resolve_crate(c, pcs));
		});
		for r in shared::deactivate_threads(ts) {
			r?;
		}

		Ok(())
	}

	fn resolve_crate(self, id: PackageId, problem_crates: Vec<ProblemCrate>) -> Result<()> {
		let p = manifest_path_of(&id.repr);
		let s = fs::read_to_string(&p)?;
		let mut d = s.parse::<DocumentMut>()?;
		// Introduce initial state to fix:
		// ```diff
		// -runtime-benchmarks = []
		// +runtime-benchmarks = [
		// + "frame-support/runtime-benchmarks",
		// +,
		// + "frame-system/runtime-benchmarks",
		// + "pallet-evm/runtime-benchmarks"]
		// ```
		let mut fs_initial_state = FxHashMap::default();

		for (i, pc) in problem_crates.iter().enumerate() {
			match &pc.problem {
				Problem::DefaultFeaturesEnabled => continue,
				Problem::MissingFeatures(fs) => fs.iter().for_each(|f| {
					let fs = d["features"].as_table_mut().unwrap();
					let fs = fs[f].as_array_mut().unwrap();

					if !fs_initial_state.contains_key(f) {
						fs_initial_state.insert(f.to_owned(), fs.is_empty());
					}

					fs.push_formatted(Value::from(format!("{}/{f}", pc.alias)).decorated(
						INDENTATION.get().unwrap(),
						if *fs_initial_state.get(f).unwrap() && i == problem_crates.len() - 1 {
							",\n"
						} else {
							""
						},
					));
				}),
			}
		}

		if *SORT.get().unwrap() {
			SortVisitor(FEATURES.get().unwrap().to_owned()).visit_document_mut(&mut d);
		}

		match MODE.get().unwrap() {
			Mode::Check => (),
			Mode::DryRun => println!("{id}\n{}", util::diff(&s, &d.to_string())),
			m => {
				let p_tmp = tmp_path_of(&p);
				let f_tmp = File::create(&p_tmp)?;
				let mut w = BufWriter::new(f_tmp);

				w.write_all(d.to_string().as_bytes())?;

				if *m == Mode::Overwrite {
					fs::rename(p_tmp, p)?;
				}
			},
		}

		Ok(())
	}
}

fn manifest_path_of(s: &str) -> String {
	format!("{}/Cargo.toml", &PATH_REGEX.captures(s).unwrap()[1])
}

fn tmp_path_of(p: &str) -> String {
	format!("{p}.cargo-featalign.swap")
}
