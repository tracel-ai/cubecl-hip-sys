#![allow(ambiguous_glob_reexports)]
#![allow(clashing_extern_declarations)]
#![allow(dead_code)]
#![allow(suspicious_runtime_symbol_definitions)]
#![allow(unused_imports)]

#[cfg(feature = "hip_41134")]
mod bindings_41134 {
    include!(concat!(env!("OUT_DIR"), "/bindings_41134.rs"));
}
#[cfg(feature = "hip_41134")]
pub use bindings_41134::*;

#[cfg(feature = "hip_42131")]
mod bindings_42131 {
    include!(concat!(env!("OUT_DIR"), "/bindings_42131.rs"));
}
#[cfg(feature = "hip_42131")]
pub use bindings_42131::*;

#[cfg(feature = "hip_42133")]
mod bindings_42133 {
    include!(concat!(env!("OUT_DIR"), "/bindings_42133.rs"));
}
#[cfg(feature = "hip_42133")]
pub use bindings_42133::*;

#[cfg(feature = "hip_42134")]
mod bindings_42134 {
    include!(concat!(env!("OUT_DIR"), "/bindings_42134.rs"));
}
#[cfg(feature = "hip_42134")]
pub use bindings_42134::*;

#[cfg(feature = "hip_43482")]
mod bindings_43482 {
    include!(concat!(env!("OUT_DIR"), "/bindings_43482.rs"));
}
#[cfg(feature = "hip_43482")]
pub use bindings_43482::*;

#[cfg(feature = "hip_43483")]
mod bindings_43483 {
    include!(concat!(env!("OUT_DIR"), "/bindings_43483.rs"));
}
#[cfg(feature = "hip_43483")]
pub use bindings_43483::*;

#[cfg(feature = "hip_43484")]
mod bindings_43484 {
    include!(concat!(env!("OUT_DIR"), "/bindings_43484.rs"));
}
#[cfg(feature = "hip_43484")]
pub use bindings_43484::*;

#[cfg(feature = "hip_51831")]
mod bindings_51831 {
    include!(concat!(env!("OUT_DIR"), "/bindings_51831.rs"));
}
#[cfg(feature = "hip_51831")]
pub use bindings_51831::*;

#[cfg(feature = "hip_52802")]
mod bindings_52802 {
    include!(concat!(env!("OUT_DIR"), "/bindings_52802.rs"));
}
#[cfg(feature = "hip_52802")]
pub use bindings_52802::*;

#[cfg(feature = "hip_53211")]
mod bindings_53211 {
    include!(concat!(env!("OUT_DIR"), "/bindings_53211.rs"));
}
#[cfg(feature = "hip_53211")]
pub use bindings_53211::*;

#[cfg(feature = "hip_60850")]
mod bindings_60850 {
    include!(concat!(env!("OUT_DIR"), "/bindings_60850.rs"));
}
#[cfg(feature = "hip_60850")]
pub use bindings_60850::*;
