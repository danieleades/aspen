#[cfg(feature = "lcm")]
extern crate lcm_gen;

#[cfg(feature = "lcm")]
fn lcm_gen()
{
	use std::path::PathBuf;

	// All of our messages will be in the "msg" directory
	let mut lcm_source_dir : PathBuf = env!("CARGO_MANIFEST_DIR").into();
	lcm_source_dir.push("msg");

	// Specify when this script needs to be rerun
	println!("cargo:rerun-if-changed={}", lcm_source_dir.display());

	// Then run the generation
	lcm_gen::LcmGen::new()
	                .add_directory(lcm_source_dir)
	                .run();

	// Now make sure Cargo includes the files
	println!("cargo:include={}", std::env::var("OUT_DIR").unwrap());
}

#[cfg(not(feature = "lcm"))]
fn lcm_gen()
{
	// No-op
}

fn main()
{
	lcm_gen();
}
