// Copyright (c) 2019 King's College London created by the Software Development Team
// <http://soft-dev.org/>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, or the UPL-1.0 license <http://opensource.org/licenses/UPL>
// at your option. This file may not be copied, modified, or distributed except according to those
// terms.

//! This crate provides a simple language testing framework designed to help when you are testing
//! things like compilers and virtual machines. It allows users to embed simple tests for process
//! success/failure and for stderr/stdout inside a source file. It is loosely based on the
//! [`compiletest_rs`](https://crates.io/crates/compiletest_rs) crate, but is much simpler (and
//! hence sometimes less powerful), and designed to be used for testing non-Rust languages too.
//!
//! For example, a Rust language tester, loosely in the spirit of
//! [`compiletest_rs`](https://crates.io/crates/compiletest_rs), looks as follows:
//!
//! ```rust
//! use std::{path::PathBuf, process::Command};
//!
//! use lang_tester::LangTester;
//! use tempdir::TempDir;
//!
//! fn main() {
//!     // We use rustc to compile files into a binary: we store those binary files into `tempdir`.
//!     // This may not be necessary for other languages.
//!     let tempdir = TempDir::new("rust_lang_tester").unwrap();
//!     LangTester::new()
//!         .test_dir("examples/rust_lang_tester/lang_tests")
//!         // Only use files named `*.rs` as test files.
//!         .test_file_filter(|p| p.extension().unwrap().to_str().unwrap() == "rs")
//!         // Extract the first sequence of commented line(s) as the tests.
//!         .test_extract(|s| {
//!             Some(
//!                 s.lines()
//!                     // Skip non-commented lines at the start of the file.
//!                     .skip_while(|l| !l.starts_with("//"))
//!                     // Extract consecutive commented lines.
//!                     .take_while(|l| l.starts_with("//"))
//!                     .map(|l| &l[2..])
//!                     .collect::<Vec<_>>()
//!                     .join("\n"),
//!             )
//!         })
//!         // We have two test commands:
//!         //   * `Compiler`: runs rustc.
//!         //   * `Run-time`: if rustc does not error, and the `Compiler` tests succeed, then the
//!         //     output binary is run.
//!         .test_cmds(move |p| {
//!             // Test command 1: Compile `x.rs` into `tempdir/x`.
//!             let mut exe = PathBuf::new();
//!             exe.push(&tempdir);
//!             exe.push(p.file_stem().unwrap());
//!             let mut compiler = Command::new("rustc");
//!             compiler.args(&["-o", exe.to_str().unwrap(), p.to_str().unwrap()]);
//!             // Test command 2: run `tempdir/x`.
//!             let runtime = Command::new(exe);
//!             vec![("Compiler", compiler), ("Run-time", runtime)]
//!         })
//!         .run();
//! }
//! ```
//!
//! This defines a lang tester that uses all `*.rs` files in a given directory as test files,
//! running two test commands against them: `Compiler` (i.e. `rustc`); and `Run-time` (the compiled
//! binary).
//!
//! Users can then write test files such as the following:
//!
//! ```rust,ignore
//! // Compiler:
//! //   status: success
//! //   stderr:
//! //     warning: unused variable: `x`
//! //       ...unused_var.rs:12:9
//! //       ...
//! //
//! // Run-time:
//! //   status: success
//! //   stdout: Hello world
//! fn main() {
//!     let x = 0;
//!     println!("Hello world");
//! }
//! ```
//!
//! Test data is specified with a two-level indentation syntax: the outer most level of indentation
//! defines a test command (multiple command names can be specified, as in the above); each test
//! command can then define sub-tests for one or more of:
//!
//!   * `status: <success|failure|<int>>`, where `success` and `failure` map to platform specific
//!      notions of a command completing successfully or unsuccessfully respectively and `<int>` is
//!      a signed integer checking for a specific exit code on platforms that support it.
//!   * `stderr: [<string>]`, `stdout: [<string>]` are matched strings against a command's `stderr`
//!     or `stdout`. The special string `...` can be used as a simple wildcard: if a line consists
//!     solely of `...`, it means "match zero or more lines"; if a line begins with `...`, it means
//!     "match the remainder of the line only"; if a line ends with `...`, it means "match the
//!     start of the line only". A line may start and end with `...`. Note that `stderr`/`stdout`
//!     matches ignore leading/trailing whitespace and newlines, but are case sensitive.
//!
//! The above file thus contains 4 actual tests: the `Compiler` should succeed (e.g. return a `0`
//! exit code when run on Unix), and its `stderr` output should warn about an unused variable on
//! line 12; and the resulting binary should succeed and produce `Hello world` on `stdout`.
//!
//! Potential sub-tests that are not mentioned are not tested: for example, the above file does not
//! state whether the `Compiler`s `stdout` should have content or not (but note that the line
//! `stdout:` on its own would state that the `Compiler` should have no content at all).
//!
//! `lang_tester`'s output is deliberately similar to Rust's normal testing output. Running the
//! example `rust_lang_tester` in this crate produces the following output:
//!
//! ```text
//! $ cargo run --example=rust_lang_tester
//!    Compiling lang_tester v0.1.0 (/home/ltratt/scratch/softdev/lang_tester)
//!     Finished dev [unoptimized + debuginfo] target(s) in 3.49s
//!      Running `target/debug/examples/rust_lang_tester`
//!
//! running 4 tests
//! test lang_tests::no_main ... ok
//! test lang_tests::unknown_var ... ok
//! test lang_tests::unused_var ... ok
//! test lang_tests::exit_code ... ok
//!
//! test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
//! ```
//!
//! If you want to run a subset of tests, you can specify simple filters which use substring match
//! to run a subset of tests:
//!
//! ```text
//! $ cargo run --example=rust_lang_tester var
//!    Compiling lang_tester v0.1.0 (/home/ltratt/scratch/softdev/lang_tester)
//!     Finished dev [unoptimized + debuginfo] target(s) in 3.37s
//!      Running `target/debug/examples/rust_lang_tester var`
//!
//! running 2 tests
//! test lang_tests::unknown_var ... ok
//! test lang_tests::unused_var ... ok
//!
//! test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out
//! ```
//!
//! Users will often want to integrate such tests into their test suite. An easy way of doing this
//! is to add a `[[test]]` entry to your `Cargo.toml` along the following lines:
//!
//! ```text
//! [[test]]
//! name = "lang_tests"
//! path = "lang_tests/run_tests.rs"
//! harness = false
//! ```
//!
//! Running `cargo test` will now also run your lang tests.

#![allow(clippy::new_without_default)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::type_complexity)]

mod fuzzy;
mod parser;
mod tester;

pub use tester::LangTester;

pub(crate) fn fatal(msg: &str) -> ! {
    eprintln!("\nFatal exception:\n  {}", msg);
    std::process::exit(1);
}
