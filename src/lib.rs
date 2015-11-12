//! R2Pipe provides functions to interact with [radare2](http://rada.re/r/)
//!
//! Hence this requires you to have radare2 installed on you system. For more
//! information refer to the r2 [repository](https://github.com/radare/radare2).
//! The module spawns an instance of r2 and communicates with it over pipes.
//! Using commands which produce a JSON output is recommended and easier to
//! parse.
//!
//! R2Pipes are available for a several of languages. For more information
//! about r2pipes in general head over to the
//! [wiki](https://github.com/radare/radare2/wiki/R2PipeAPI).
//!
//! # Design
//! All the functionality for the crate are exposed through two structs:
//! `R2PipeLang` and `R2PipeSpawn`.
//!
//! Typically, there are two ways to invoke r2pipe. One by spawning a
//! child-process
//! from inside r2 and second by making the program spawn a child r2process.
//! `enum R2Pipe` is provided to allow easier use of the library and abstract
//! the
//! difference between these two methods.
//!
//! The `macro open_pipe!()` determines which of the two methods to use.
//!
//! **Note:** For the second method,
//! the path of the executable to be analyzed must be provided, while this is
//! implicit in the first (pass `None`) method (executable loaded by r2).
//!
//! # Example
//! ```no_run
//! #[macro_use]
//! extern crate r2pipe;
//! use r2pipe::R2Pipe;
//! fn main() {
//!     let path = Some("/bin/ls".to_owned());
//!     let mut r2p = open_pipe!(path).unwrap();
//!     println!("{}", r2p.cmd("?e Hello World").unwrap());
//!     if let Ok(json) = r2p.cmdj("ij") {
//!         println!("{}", json.pretty());
//!         println!("ARCH {}", json.find_path(&["bin","arch"]).unwrap());
//!     }
//!     r2p.close();
//! }
//! ```

#![doc(html_root_url = "https://radare.github.io/r2pipe.rs/")]

extern crate libc;
extern crate rustc_serialize;

#[macro_use]
pub mod r2pipe;
pub mod r2;
pub mod structs;

// Rexport to bring it out one module.
pub use self::r2pipe::R2Pipe;
pub use self::r2::R2;
