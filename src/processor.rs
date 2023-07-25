// std
use std::{
	collections::HashMap,
	mem,
	path::PathBuf,
	sync::{
		atomic::{AtomicU16, Ordering},
		Arc, Mutex,
	},
	thread,
};
// crates.io
use cargo_metadata::{
	CargoOpt, DependencyKind, Metadata, MetadataCommand, Node, NodeDep, Package, PackageId, Resolve,
};
use once_cell::sync::{Lazy, OnceCell};
use serde::Serialize;
// subalfred
use crate::util::GetById;

#[allow(clippy::type_complexity)]
pub static PROBLEMS: Lazy<Arc<Mutex<HashMap<PackageId, Vec<ProblemCrate>>>>> =
	Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));
pub fn append_problems(id: PackageId, mut problems: Vec<ProblemCrate>) {
	PROBLEMS
		.lock()
		.unwrap()
		.entry(id)
		.and_modify(|pcs| pcs.append(&mut problems))
		.or_insert(problems);
}

static WORKSPACE_ONLY: OnceCell<bool> = OnceCell::new();
static DEFAULT_STD: OnceCell<bool> = OnceCell::new();
static THREAD: OnceCell<u16> = OnceCell::new();
static THREAD_ACTIVE: Lazy<AtomicU16> = Lazy::new(|| AtomicU16::new(1));

#[derive(Debug, Clone)]
pub struct Processor {
	features: Arc<Vec<String>>,
	// TODO?: replace with `HashMap` packages
	metadata: Arc<Metadata>,
	// Remove?
	resolve: Arc<Resolve>,
}
impl Processor {
	pub fn analyze(
		manifest_path: &PathBuf,
		features: Vec<String>,
		workspace_only: bool,
		default_std: bool,
		thread_count: u16,
	) -> Self {
		// These variables are initialized only once before processing,
		// so they must be `Some` in the following context.
		WORKSPACE_ONLY.set(workspace_only).unwrap();
		DEFAULT_STD.set(default_std).unwrap();
		THREAD.set(thread_count).unwrap();

		let mut metadata = MetadataCommand::new()
			.manifest_path(manifest_path)
			.features(CargoOpt::AllFeatures)
			.exec()
			.unwrap();
		let resolve = mem::take(&mut metadata.resolve).unwrap();

		Self {
			features: Arc::new(features),
			metadata: Arc::new(metadata),
			resolve: Arc::new(resolve),
		}
	}

	pub fn process(self, depth: i16) {
		let r = self.resolve.root.as_ref().expect(
			"the `[package]` specified in the `Cargo.toml` cannot be found\n\
			it appears to be a pure workspace which is not supported",
		);
		let n = self.resolve.nodes.get_by_id(r).unwrap().to_owned();
		let p = self.metadata.get_by_id(&n.id).unwrap().to_owned();

		self.process_package(n, p, depth, String::new());
	}

	fn process_package(
		self,
		node: Node,
		package: Package,
		depth: i16,
		mut dependency_path: String,
	) {
		if *WORKSPACE_ONLY.get().unwrap() && !self.is_workspace_member(&package.id) {
			return;
		}

		dependency_path.push_str(&format!("/{}", package.name.clone()));

		let rs = package
			.dependencies
			.iter()
			.filter_map(|d| d.rename.as_ref().map(|rn| (d.name.as_str(), rn.as_str())))
			.collect();

		if *DEFAULT_STD.get().unwrap() {
			self.check_default_features(&node, &package, &dependency_path);
		}

		for (f, required_fs) in &package.features {
			if self.features.contains(f) {
				self.check_features(&node, &dependency_path, f, required_fs, &rs);
			}
		}

		if in_depth(depth) {
			let mut ts = Vec::new();

			for d in &node.deps {
				if ignore(d) {
					continue;
				}

				let n = self.resolve.get_by_id(&d.pkg).unwrap().to_owned();
				let p = self.metadata.get_by_id(&d.pkg).unwrap().to_owned();
				let dependency_path = dependency_path.clone();
				let psr = self.clone();

				// TODO: optimize, take this out of the loop
				if THREAD_ACTIVE.load(Ordering::SeqCst) < *THREAD.get().unwrap() - 1 {
					ts.push(thread::spawn(move || {
						psr.process_package(n, p, depth - 1, dependency_path)
					}));

					THREAD_ACTIVE.fetch_add(1, Ordering::SeqCst);
				} else {
					psr.process_package(n, p, depth - 1, dependency_path);
				}
			}

			let ts_count = ts.len() as u16;

			for t in ts {
				t.join().unwrap();
			}

			THREAD_ACTIVE.fetch_sub(ts_count, Ordering::SeqCst);
		}
	}

	fn check_default_features(&self, node: &Node, package: &Package, dependency_path: &str) {
		let mut problem_cs = Vec::new();

		// The items we require are separated between two vectors: `node.deps` and
		// `package.dependencies`.
		for d in &node.deps {
			if ignore(d) {
				continue;
			}

			let p = self.metadata.get_by_id(&d.pkg).unwrap();

			if package.dependencies.iter().any(|d| {
				d.name == p.name
					&& d.uses_default_features
					&& p.features
						.get("default")
						.map(|dfs| dfs.iter().any(|f| f == "std"))
						.unwrap_or_default()
			}) {
				problem_cs.push(ProblemCrate {
					id: p.id.clone(),
					dependency_path: dependency_path.to_owned(),
					problem: Problem::DefaultFeaturesEnabled,
				});
			}
		}

		append_problems(node.id.clone(), problem_cs);
	}

	fn check_features(
		&self,
		node: &Node,
		dependency_path: &str,
		feature: &str,
		required_features: &[String],
		renames: &HashMap<&str, &str>,
	) {
		let mut problem_cs = Vec::new();

		for d in &node.deps {
			if ignore(d) {
				continue;
			}

			let p_id = &d.pkg;
			let p = self.metadata.get_by_id(p_id).unwrap();
			let p_name = p.name.as_str();
			let p_rename = renames.get(p_name).copied().unwrap_or(p_name);
			let n = self.resolve.get_by_id(p_id).unwrap();

			// If the dependency has the feature specified by the user for processing.
			if n.features.iter().any(|f| f == feature) {
				let mut err = true;

				// `assert!("general-a/std".contains("general-a"));`
				for f in required_features {
					// TODO: handle the full name here
					// e.g. this could be `general-a/std` or `general-a?/std`
					if f.contains(p_rename) {
						err = false;

						break;
					}
				}

				if err {
					problem_cs.push(ProblemCrate {
						id: p_id.to_owned(),
						dependency_path: dependency_path.to_owned(),
						problem: Problem::MissingFeature(feature.into()),
					});
				}
			}
		}

		if !problem_cs.is_empty() {
			append_problems(node.id.clone(), problem_cs);
		}
	}

	fn is_workspace_member(&self, id: &PackageId) -> bool {
		self.metadata.workspace_members.contains(id)
	}
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProblemCrate {
	id: PackageId,
	dependency_path: String,
	problem: Problem,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Problem {
	DefaultFeaturesEnabled,
	MissingFeature(String),
}

// Ignore `[dev-dependencies]`.
fn ignore(node_dep: &NodeDep) -> bool {
	node_dep.dep_kinds.iter().any(|k| matches!(k.kind, DependencyKind::Development))
}

fn in_depth(depth: i16) -> bool {
	depth != 0 || depth == -1
}
