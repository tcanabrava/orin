// SPDX-License-Identifier: MIT

//! Every screen outside gameplay: the page state machine and routing, the\n//! shared page scaffolding, and one file per page. The top of the library\n//! stack — it registers the editor and reaches jam, so nothing may depend\n//! on it but the binary.

pub mod menu;
pub use menu::*;
