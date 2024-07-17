// crates.io
use vergen_gitcl::{CargoBuilder, Emitter, GitclBuilder};

fn main() {
	let mut emitter = Emitter::default();

	emitter
		.add_instructions(&CargoBuilder::default().target_triple(true).build().unwrap())
		.unwrap();

	// Disable the git version if installed from [crates.io](https://crates.io).
	if emitter.add_instructions(&GitclBuilder::default().sha(true).build().unwrap()).is_err() {
		println!("cargo:rustc-env=VERGEN_GIT_SHA=crates.io");
	}

	emitter.emit().unwrap();
}
