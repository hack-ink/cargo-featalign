// std
use std::mem;
// crates.io
use toml_edit::{visit_mut::VisitMut, Formatted, RawString, Table, Value};
// cargo-featalign
use crate::shared::INDENTATION;

#[derive(Debug)]
pub struct SortVisitor(pub Vec<String>);
impl VisitMut for SortVisitor {
	fn visit_table_mut(&mut self, node: &mut Table) {
		fn sort(mut v: Vec<Formatted<String>>) -> Vec<Formatted<String>> {
			fn has_empty_prefix(formatted_string: &Formatted<String>) -> bool {
				formatted_string
					.decor()
					.prefix()
					.and_then(|s| s.as_str())
					.map(|s| s.trim().is_empty())
					.unwrap_or_default()
			}

			fn sort_sub(v: &mut Vec<Formatted<String>>, prefix: Option<RawString>) {
				v.sort_by(|a, b| a.value().cmp(b.value()));

				if let Some(p) = prefix {
					v[0].decor_mut().set_prefix(p);
				}
			}

			let mut offset = 0;
			let mut pivots = Vec::new();

			v.iter_mut().enumerate().for_each(|(i, s)| {
				if !has_empty_prefix(s) {
					let d = s.decor_mut();
					let p = d.prefix().unwrap().to_owned();

					pivots.push((i - offset, p));

					offset = i;
				}

				s.decor_mut().clear();
			});

			let mut prefix = None;
			let mut v_chunks = Vec::new();

			for (i, p) in pivots {
				let v_chunk = v.split_off(i);

				if !v.is_empty() {
					sort_sub(&mut v, prefix.take());

					v_chunks.push(mem::take(&mut v));
				}

				prefix = Some(p);
				v = v_chunk;
			}

			sort_sub(&mut v, prefix.take());

			v_chunks.push(v);

			v_chunks.concat()
		}

		if let Some(v) = node.get_mut("features") {
			let t = v.as_table_mut().unwrap();
			let pfs = mem::take(&mut self.0);

			pfs.into_iter().for_each(|f| {
				if let Some(rfs) = t.get_mut(&f) {
					let rfs = rfs.as_array_mut().unwrap();
					let rfs_ = mem::take(rfs);

					rfs.set_trailing(rfs_.trailing().to_owned());

					let rfs_values = rfs_
						.into_iter()
						.map(|v| if let Value::String(s) = v { s } else { unreachable!() })
						.collect::<Vec<_>>();

					sort(rfs_values).into_iter().for_each(|f| {
						let v = if f.decor().prefix().is_none() {
							Value::String(f).decorated(INDENTATION.get().unwrap(), "")
						} else {
							Value::String(f)
						};

						rfs.push_formatted(v);
					});

					if !rfs.is_empty() {
						rfs.set_trailing_comma(true);

						if rfs.trailing().as_str().map(|s| s.is_empty()).unwrap_or(true) {
							rfs.set_trailing("\n");
						}
					}
				}
			});
		}
	}
}
