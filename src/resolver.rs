// std
use std::{
	fs::{self, File},
	io::{BufWriter, Write},
	mem,
};
// crates.io
use cargo_metadata::PackageId;
use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use toml_edit::{Document, Value};
// cargo-featalign
use crate::{
	analyzer::{Problem, ProblemCrate, PROBLEMS},
	cli::{IndentSymbol, Mode, ResolverInitiator},
	prelude::*,
	shared::MODE,
};

static PATH_REGEX: Lazy<Regex> =
	Lazy::new(|| Regex::new(r".+? \d.\d.\d \(path\+file://(/.+?)\)").unwrap());
static INDENTATION: OnceCell<String> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct Resolver;
impl Resolver {
	pub fn initialize(initiator: ResolverInitiator) -> Self {
		let indentation = match initiator.indent_symbol {
			IndentSymbol::Tab => "\n\t".into(),
			IndentSymbol::Whitespace => format!("\n{}", " ".repeat(initiator.indent_size)),
		};

		INDENTATION.set(indentation).unwrap();

		Self
	}

	pub fn resolve(self) -> Result<()> {
		if *MODE.get().unwrap() == Mode::Check {
			return Ok(());
		}

		let ps = mem::take(&mut *PROBLEMS.lock().unwrap());
		let mut ts = Vec::new();

		for (c, pcs) in ps {
			let r = self.clone();

			shared::activate_thread(&mut ts, move || r.resolve_crate(c, pcs));
		}
		for r in shared::deactivate_threads(ts) {
			r?;
		}

		Ok(())
	}

	fn resolve_crate(self, id: PackageId, problem_crates: Vec<ProblemCrate>) -> Result<()> {
		let p = manifest_path_of(&id.repr);
		let s = fs::read_to_string(&p)?;
		let mut d = s.parse::<Document>()?;

		for pc in problem_crates {
			match pc.problem {
				Problem::DefaultFeaturesEnabled => continue,
				Problem::MissingFeatures(fs) =>
					for f in fs {
						let fs = d["features"].as_table_mut().unwrap();
						let fs = fs[&f].as_array_mut().unwrap();

						fs.push_formatted(Value::from(format!("{}/{f}", pc.alias)).decorated(
							INDENTATION.get().unwrap(),
							if fs.len() == 0 { ",\n" } else { "" },
						));
					},
			}
		}

		match MODE.get().unwrap() {
			Mode::Check => (),
			Mode::DryRun => {
				println!("{id}\n{}", util::diff(&s, &d.to_string()));
			},
			m @ Mode::DryRun2 | m @ Mode::Overwrite => {
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
