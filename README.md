<div align="center">

# Cargo Featalign
### Cargo features alignment tool.

[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Checks](https://github.com/hack-ink/cargo-featalign/actions/workflows/checks.yml/badge.svg?branch=main)](https://github.com/hack-ink/cargo-featalign/actions/workflows/checks.yml)
[![Release](https://github.com/hack-ink/cargo-featalign/actions/workflows/release.yml/badge.svg)](https://github.com/hack-ink/cargo-featalign/actions/workflows/release.yml)
[![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/hack-ink/cargo-featalign)](https://github.com/hack-ink/cargo-featalign/tags)
[![GitHub code lines](https://tokei.rs/b1/github/hack-ink/cargo-featalign)](https://github.com/hack-ink/cargo-featalign)
[![GitHub last commit](https://img.shields.io/github/last-commit/hack-ink/cargo-featalign?color=red&style=plastic)](https://github.com/hack-ink/cargo-featalign)

</div>

### Introduction
The original version of this project can be found at [`subalfred check features`](https://github.com/hack-ink/subalfred).
Upon further investigation, I have found that this tool is not only compatible with *Substrate* projects but also works for general *Cargo* projects, offering even more powerful features than before.
Now, `cargo-featalign` stands out with its enhanced functionality.

The `cargo-featalign` tool offers the following features:
- Checking for missing features
- Printing the dependency path
- Performing a dry run before overwriting
- Automatically aligning/fixing missing features
- Sorting alphabetically while aligning

### Installation
- From GitHub: [`github.com/hack-ink/cargo-featalign/releases/latest`](https://github.com/hack-ink/**cargo-featalign/releases/latest)
- From Cargo: `cargo install cargo-featalign`

### Usage
```sh
cargo featalign --help
```
```
Cargo features alignment tool.

Usage: cargo-featalign [OPTIONS] --features <[NAME]> [PATH]

Arguments:
  [PATH]
          Root `Cargo.toml`'s path.

          If `Cargo.toml` is not provided, it will be searched for under the specified path.

          [default: ./Cargo.toml]

Options:
      --features <[NAME]>
          Features to process

      --thread <NUM>
          Number of threads to use.

          The default value is based on the number of logical cores.

          [default: 32]

      --mode <MODE>
          Running mode.

          Check: Prints the analysis result.
          DryRun: Prints the resolved result without modifying the `Cargo.toml` file.
          DryRun2: creates a `*.cargo-featalign.swap` file.
          Overwrite: Overwrites the original `Cargo.toml` file.

          [default: overwrite]
          [possible values: check, dry-run, dry-run2, overwrite]

      --indent-symbol <INDENT_SYMBOL>
          Use the given symbol for indentation

          [default: tab]
          [possible values: tab, whitespace]

      --indent-size <SIZE>
          The number of spaces used for indentation

          [default: 4]

      --workspace-only
          Determines whether to process only workspace members

      --default-std
          Determines whether to check default features.

          This option is useful when working in a no-std environment. This feature checks if you have set `default-features = false` while also having a `std = ["x/std"]` part to control it separately.

      --depth <NUM>
          Depth of the dependency tree to process.

          Use `-1` to process the entire tree.

          !! Running with this flag under a large project, even with 128 threads configured, is incredibly challenging.

          [default: 0]

      --sort
          Wether to sort the required features while aligning

      --verbose
          Verbose output

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Example
#### Preparation
```sh
cargo install cargo-featalign
git clone https://github.com/hack-ink/cargo-featalign.git
cd cargo-featalign
```

#### Only check the features of top-level workspace members
```sh
cargo featalign mock --features std,runtime-benchmarks,try-runtime --workspace-only --default-std --depth -1 --mode check | jq
```
```json
{
  "mock-runtime 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock)": [
    {
      "id": "general-c 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/general/c)",
      "alias": "",
      "dependency-path": "/mock-runtime",
      "problem": "default-features-enabled"
    },
    {
      "id": "pallet-a 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/pallet/a)",
      "alias": "pallet-a",
      "dependency-path": "/mock-runtime",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    },
    {
      "id": "pallet-b 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/pallet/b)",
      "alias": "pallet-b",
      "dependency-path": "/mock-runtime",
      "problem": {
        "missing-features": [
          "runtime-benchmarks"
        ]
      }
    },
    {
      "id": "pallet-c 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/pallet/c)",
      "alias": "pallet-c",
      "dependency-path": "/mock-runtime",
      "problem": {
        "missing-features": [
          "try-runtime"
        ]
      }
    },
    {
      "id": "pallet-d 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/pallet/d)",
      "alias": "pallet-d",
      "dependency-path": "/mock-runtime",
      "problem": {
        "missing-features": [
          "runtime-benchmarks",
          "std",
          "try-runtime"
        ]
      }
    }
  ]
}
```


#### Check the features of workspace members recursively
```sh
cargo featalign mock --features std,runtime-benchmarks,try-runtime --workspace-only --default-std --depth -1 --mode check | jq
```
```json
{
  "nested-a 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/nested/a)": [
    {
      "id": "nested-d 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/nested/d)",
      "alias": "",
      "dependency-path": "/mock-runtime/primitive-a/nested-a",
      "problem": "default-features-enabled"
    },
    {
      "id": "nested-b 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/nested/b)",
      "alias": "nested-b",
      "dependency-path": "/mock-runtime/primitive-a/nested-a",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    }
  ],
  "nested-b 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/nested/b)": [
    {
      "id": "nested-c 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/nested/c)",
      "alias": "nested-c",
      "dependency-path": "/mock-runtime/primitive-a/nested-a/nested-b",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    }
  ],
  "mock-runtime 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock)": [
    {
      "id": "general-c 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/general/c)",
      "alias": "",
      "dependency-path": "/mock-runtime",
      "problem": "default-features-enabled"
    },
    {
      "id": "pallet-a 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/pallet/a)",
      "alias": "pallet-a",
      "dependency-path": "/mock-runtime",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    },
    {
      "id": "pallet-b 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/pallet/b)",
      "alias": "pallet-b",
      "dependency-path": "/mock-runtime",
      "problem": {
        "missing-features": [
          "runtime-benchmarks"
        ]
      }
    },
    {
      "id": "pallet-c 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/pallet/c)",
      "alias": "pallet-c",
      "dependency-path": "/mock-runtime",
      "problem": {
        "missing-features": [
          "try-runtime"
        ]
      }
    },
    {
      "id": "pallet-d 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/pallet/d)",
      "alias": "pallet-d",
      "dependency-path": "/mock-runtime",
      "problem": {
        "missing-features": [
          "runtime-benchmarks",
          "std",
          "try-runtime"
        ]
      }
    }
  ]
}
```

#### Check the features of all dependencies recursively
**!! Running this under a large project, even with 128 threads configured, is incredibly challenging.**
```sh
cargo featalign . --features std --depth -1 --mode check | jq
```
```json
{
  "semver 1.0.18 (registry+https://github.com/rust-lang/crates.io-index)": [
    {
      "id": "serde 1.0.176 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "serde",
      "dependency-path": "/cargo-featalign/cargo_metadata/semver",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    }
  ],
  "tracing-core 0.1.31 (registry+https://github.com/rust-lang/crates.io-index)": [
    {
      "id": "valuable 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "valuable",
      "dependency-path": "/cargo-featalign/color-eyre/color-spantrace/tracing-core",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    },
    {
      "id": "valuable 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "valuable",
      "dependency-path": "/cargo-featalign/color-eyre/tracing-error/tracing/tracing-core",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    },
    {
      "id": "valuable 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "valuable",
      "dependency-path": "/cargo-featalign/color-eyre/tracing-error/tracing-subscriber/tracing-core",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    },
    {
      "id": "valuable 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "valuable",
      "dependency-path": "/cargo-featalign/color-eyre/color-spantrace/tracing-error/tracing/tracing-core",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    },
    {
      "id": "valuable 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "valuable",
      "dependency-path": "/cargo-featalign/color-eyre/color-spantrace/tracing-error/tracing-subscriber/tracing-core",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    }
  ],
  "backtrace 0.3.68 (registry+https://github.com/rust-lang/crates.io-index)": [
    {
      "id": "libc 0.2.147 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "libc",
      "dependency-path": "/cargo-featalign/color-eyre/backtrace",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    }
  ],
  "rustix 0.38.4 (registry+https://github.com/rust-lang/crates.io-index)": [
    {
      "id": "libc 0.2.147 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "libc",
      "dependency-path": "/cargo-featalign/clap/clap_builder/anstream/is-terminal/rustix",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    }
  ],
  "errno 0.3.1 (registry+https://github.com/rust-lang/crates.io-index)": [
    {
      "id": "libc 0.2.147 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "libc",
      "dependency-path": "/cargo-featalign/clap/clap_builder/anstream/is-terminal/rustix/errno",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    }
  ],
  "time 0.3.23 (registry+https://github.com/rust-lang/crates.io-index)": [
    {
      "id": "libc 0.2.147 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "libc",
      "dependency-path": "/cargo-featalign/vergen/time",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    },
    {
      "id": "serde 1.0.176 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "serde",
      "dependency-path": "/cargo-featalign/vergen/time",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    }
  ],
  "ahash 0.8.3 (registry+https://github.com/rust-lang/crates.io-index)": [
    {
      "id": "once_cell 1.18.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "once_cell",
      "dependency-path": "/cargo-featalign/imara-diff/ahash",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    }
  ],
  "getrandom 0.2.10 (registry+https://github.com/rust-lang/crates.io-index)": [
    {
      "id": "libc 0.2.147 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "libc",
      "dependency-path": "/cargo-featalign/imara-diff/ahash/getrandom",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    }
  ],
  "clap 4.3.19 (registry+https://github.com/rust-lang/crates.io-index)": [
    {
      "id": "once_cell 1.18.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "alias": "once_cell",
      "dependency-path": "/cargo-featalign/clap",
      "problem": {
        "missing-features": [
          "std"
        ]
      }
    }
  ]
}
```

#### Dry run of aligning features for workspace members
```sh
cargo featalign mock --features std,runtime-benchmarks,try-runtime --workspace-only --default-std --depth -1 --mode dry-run
```
```diff
nested-a 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/nested/a)
@@ -17,4 +17,5 @@
 default = ["std"]
 std = [
    "nested-d/std",
+   "nested-b/std",
 ]

nested-b 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/nested/b)
@@ -14,4 +14,6 @@

 [features]
 default = ["std"]
-std     = []
+std     = [
+   "nested-c/std",
+]

mock-runtime 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock)
@@ -45,18 +45,24 @@
    "pallet-c/std",
    # "pallet-d/std",
    "primitive-a/std",
+   "pallet-a/std",
+   "pallet-d/std",
 ]

 runtime-benchmarks = [
    "pallet-a/runtime-benchmarks",
    # "pallet-b/runtime-benchmarks",
    "pallet-c/runtime-benchmarks",
+   "pallet-b/runtime-benchmarks",
+   "pallet-d/runtime-benchmarks",
    # "pallet-d/runtime-benchmarks",
 ]

 try-runtime = [
    "pallet-a/try-runtime",
    "pallet-b/try-runtime",
+   "pallet-c/try-runtime",
+   "pallet-d/try-runtime",
    # "pallet-c/try-runtime",
    # "pallet-d/try-runtime",
 ]
```

#### Dry run V2 of aligning features for workspace members
```sh
cargo featalign mock --features std,runtime-benchmarks,try-runtime --workspace-only --default-std --depth -1 --mode dry-run2
```
```sh
diff mock/Cargo.toml mock/Cargo.toml.cargo-featalign.swap
```

#### Sorting
```sh
cargo featalign mock --features std,runtime-benchmarks,try-runtime --workspace-only --default-std --depth -1 --mode dry-run --sort
```
```diff
@@ -16,5 +16,6 @@
 [features]
 default = ["std"]
 std = [
+   "nested-b/std",
    "nested-d/std",
 ]

nested-b 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock/nested/b)
@@ -14,4 +14,6 @@

 [features]
 default = ["std"]
-std     = []
+std     = [
+   "nested-c/std",
+]

mock-runtime 0.0.0 (path+file:///root/code/hack-ink/cargo-featalign/mock)
@@ -48,24 +48,34 @@
    "pallet-b/std",
    "pallet-c/std",
    # "pallet-d/std",
+   "pallet-a/std",
+   "pallet-d/std",
    "primitive-a/std",
 ]

 runtime-benchmarks = [
    "pallet-a/runtime-benchmarks",
    # "pallet-b/runtime-benchmarks",
+   "pallet-b/runtime-benchmarks",
    "pallet-c/runtime-benchmarks",
+   "pallet-d/runtime-benchmarks",
    # "pallet-d/runtime-benchmarks",
 ]

 try-runtime = [
    "pallet-a/try-runtime",
    "pallet-b/try-runtime",
+   "pallet-c/try-runtime",
+   "pallet-d/try-runtime",
    # "pallet-c/try-runtime",
    # "pallet-d/try-runtime",
 ]

-empty = []
+empty = [
+   "primitive-b/empty",
+   "primitive-c/empty",
+   "primitive-d/empty",
+]

 [workspace]
 resolver = "2"
```
