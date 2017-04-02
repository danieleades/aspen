//! Contains the LCM message type used to represent a node.

#![allow(missing_docs)]

// If there is more than one message that is generated and/or you
// want to preserve the mod heirarchy, there is a compiler plugin
// called mod_path! which would be very useful.
include!(concat!(env!("OUT_DIR"), "/node_message/mod.rs"));
