// std
use std::{
	fs::{self, File},
	io::{BufWriter, Write},
	mem, thread,
};
// crates.io
use once_cell::sync::Lazy;
use regex::Regex;
use toml_edit::{Document, Value};
// cargo-featalign
use crate::{
	analyzer::{Problem, PROBLEMS},
	prelude::*,
};

static PATH_REGEX: Lazy<Regex> =
	Lazy::new(|| Regex::new(r".+? \d.\d.\d \(path\+file://(/.+?)\)").unwrap());

pub fn resolve() -> Result<()> {
	let ps = mem::take(&mut *PROBLEMS.lock().unwrap());
	// let mut ts = Vec::new();

	for (c, pcs) in ps {
		let p = manifest_path_of(&c.repr);
		let c = fs::read_to_string(&p)?;
		let mut d = c.parse::<Document>()?;

		for pc in pcs {
			match pc.problem {
				Problem::DefaultFeaturesEnabled => continue,
				Problem::MissingFeatures(fs) =>
					for f in fs {
						let fs = d["features"].as_table_mut().unwrap();
						let fs = fs[&f].as_array_mut().unwrap();

						fs.push_formatted(Value::from(format!("{}/{f}", pc.alias)));
					},
			}
		}

		let p_tmp = tmp_path_of(&p);
		let f_tmp = File::create(&p_tmp)?;
		let mut w = BufWriter::new(f_tmp);

		w.write_all(d.to_string().as_bytes())?;
	}

	Ok(())
}

fn manifest_path_of(s: &str) -> String {
	format!("{}/Cargo.toml", &PATH_REGEX.captures(s).unwrap()[1])
}

fn tmp_path_of(p: &str) -> String {
	format!("{p}.cargo-featalign.swap")
}
