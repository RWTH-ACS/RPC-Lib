// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod constant;
mod datatype;
mod declaration;
mod enumdef;
mod parser;
mod procedure;
mod program;
mod structdef;
mod typedef;
mod uniondef;
mod xdr_spec;

pub use parser::parse;
