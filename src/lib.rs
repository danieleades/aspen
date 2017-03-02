/// Wraps items in a block for ease of applying attributes
macro_rules! block { ($($thing:item)*) => ($($thing)*) }

#[cfg(feature = "messages")]
block!{
	extern crate lcm;

	pub use node_message::NodeMsg;

	mod node_message
	{
		// If there is more than one message that is generated and/or you
		// want to preserve the mod heirarchy, there is a compiler plugin
		// called mod_path! which would be very useful.
		include!(concat!(env!("OUT_DIR"), "/node_message/mod.rs"));
	}
}

#[cfg(test)]
mod test;

pub use status::Status;
pub use node::Node;
pub use bt::BehaviorTree;

mod status;
mod node;
mod bt;

pub mod std_nodes;
