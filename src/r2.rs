// Copyright (c) 2015, The Radare Project. All rights reserved.
// See the COPYING file at the top-level directory of this distribution.
// Licensed under the BSD 3-Clause License:
// <http://opensource.org/licenses/BSD-3-Clause>
// This file may not be copied, modified, or distributed
// except according to those terms.

//! `R2` struct to make interaction with r2 easier
//!
//! This module is for convenience purposes. It provides a nicer way to
//! interact with r2 by
//! parsing the JSON into structs. Note that all commands are not supported and
//! this
//! module is updated only on a need basis. r2 commands that are not supported
//! by this module can
//! still be called by using the send() and recv() that `R2` provides. If this
//! is a command that
//! you see yourself using frequently and feel it is important to have nice
//! rust wrappers
//! feel free to raise an issue, or better yet a pull request implementing the
//! same.

use r2pipe::R2Pipe;
use serde_json;
use serde_json::Error;
use serde_json::Value;

use super::structs::*;

mod t_structs {
    use structs::{FunctionInfo, LCallInfo};

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct FunctionInfo_ {
        pub callrefs: Option<Vec<LCallInfo>>,
        pub calltype: Option<String>,
        pub codexrefs: Option<Vec<LCallInfo>>,
        pub datarefs: Option<Vec<u64>>,
        pub dataxrefs: Option<Vec<u64>>,
        pub name: Option<String>,
        pub offset: Option<u64>,
        pub realsz: Option<u64>,
        pub size: Option<u64>,
        pub ftype: Option<String>,
    }

    impl<'a> From<&'a FunctionInfo_> for FunctionInfo {
        fn from(finfo: &'a FunctionInfo_) -> FunctionInfo {
            FunctionInfo {
                callrefs: finfo.callrefs.clone(),
                calltype: finfo.calltype.clone(),
                codexrefs: finfo.codexrefs.clone(),
                datarefs: finfo.datarefs.clone(),
                dataxrefs: finfo.dataxrefs.clone(),
                name: finfo.name.clone(),
                offset: finfo.offset.clone(),
                realsz: finfo.realsz.clone(),
                size: finfo.size.clone(),
                ftype: finfo.ftype.clone(),
                locals: None,
            }
        }
    }
}

pub struct R2 {
    pipe: R2Pipe,
    readin: String,
}

impl Default for R2 {
    fn default() -> R2 {
        R2::new::<&str>(None).expect("Unable to spawn r2 or find an open r2pipe")
    }
}

// fn send and recv allow users to send their own commands,
// i.e. The ones that are not currently abstracted by the R2 API.
// Ideally, all commonly used commands must be supported for easier use.
impl R2 {
    // TODO: Use an error type
    pub fn new<T: AsRef<str>>(path: Option<T>) -> Result<R2, String> {
        if path.is_none() && !R2::in_session() {
            let e = "No r2 session open. Please specify path!".to_owned();
            return Err(e);
        }

        // This means that path is `Some` or we have an open session.
        let pipe = open_pipe!(path.as_ref()).unwrap();
        Ok(R2 {
            pipe: pipe,
            readin: String::new(),
        })
    }

    pub fn in_session() -> bool {
        match R2Pipe::in_session() {
            Some(_) => true,
            None => false,
        }
    }

    pub fn from(r2p: R2Pipe) -> R2 {
        R2 {
            pipe: r2p,
            readin: String::new(),
        }
    }

    // Does some basic configurations (sane defaults).
    pub fn init(&mut self) {
        self.send("e asm.esil = true");
        self.send("e scr.color = false");
        self.analyze();
    }

    pub fn close(&mut self) {
        self.send("q!");
    }

    pub fn send(&mut self, cmd: &str) {
        self.readin = self.pipe.cmd(cmd).unwrap();
    }

    pub fn recv(&mut self) -> String {
        let res = self.readin.clone();
        self.flush();
        res
    }

    pub fn recv_json(&mut self) -> Value {
        let mut res = self.recv().replace("\n", "");
        if res.is_empty() {
            res = "{}".to_owned();
        }

        // TODO: this should probably return a Result<Value, Error>
        serde_json::from_str(&res).unwrap()
    }

    pub fn flush(&mut self) {
        self.readin = String::from("");
    }

    pub fn analyze(&mut self) {
        self.send("aa");
        self.flush();
    }

    pub fn function(&mut self, func: &str) -> Result<LFunctionInfo, Error> {
        let cmd = format!("pdfj @ {}", func);
        self.send(&cmd);
        let raw_json = self.recv();
        // Handle Error here.
        serde_json::from_str(&raw_json)
    }

    // get 'n' (or 16) instructions at 'offset' (or current position if offset in
    // `None`)
    pub fn insts(&mut self, n: Option<u64>, offset: Option<&str>) -> Result<Vec<LOpInfo>, Error> {
        let n = n.unwrap_or(16);
        let offset: &str = offset.unwrap_or_default();
        let mut cmd = format!("pdj{}", n);
        if !offset.is_empty() {
            cmd = format!("{} @ {}", cmd, offset);
        }
        self.send(&cmd);
        let raw_json = self.recv();
        serde_json::from_str(&raw_json)
    }

    pub fn reg_info(&mut self) -> Result<LRegInfo, Error> {
        self.send("drpj");
        let raw_json = self.recv();
        serde_json::from_str(&raw_json)
    }

    pub fn flag_info(&mut self) -> Result<Vec<LFlagInfo>, Error> {
        self.send("fj");
        let raw_json = self.recv();
        serde_json::from_str(&raw_json)
    }

    pub fn bin_info(&mut self) -> Result<LBinInfo, Error> {
        self.send("ij");
        let raw_json = self.recv();
        serde_json::from_str(&raw_json)
    }

    pub fn fn_list(&mut self) -> Result<Vec<FunctionInfo>, Error> {
        self.send("aflj");
        let raw_json = self.recv();
        let mut finfo: Result<Vec<FunctionInfo>, Error> =
            serde_json::from_str::<Vec<t_structs::FunctionInfo_>>(&raw_json)
                .map(|x| x.iter().map(From::from).collect());
        if let Ok(ref mut fns) = finfo {
            for f in fns.iter_mut() {
                let res = self.locals_of(f.offset.unwrap());
                if res.is_ok() {
                    f.locals = res.ok();
                } else {
                    f.locals = Some(Vec::new());
                }
            }
        }
        finfo
    }

    pub fn sections(&mut self) -> Result<Vec<LSectionInfo>, Error> {
        self.send("Sj");
        serde_json::from_str(&self.recv())
    }

    pub fn strings(&mut self, data_only: bool) -> Result<Vec<LStringInfo>, Error> {
        if data_only {
            self.send("izj");
            serde_json::from_str(&self.recv())
        } else {
            self.send("izzj");
            let x: Result<Vec<LStringInfo>, Error> = serde_json::from_str(&self.recv());
            x
        }
    }

    pub fn locals_of(&mut self, location: u64) -> Result<Vec<LVarInfo>, Error> {
        self.send(&format!("afvbj @ {}", location));
        let x: Result<Vec<LVarInfo>, Error> = serde_json::from_str(&self.recv());
        x
    }
}
