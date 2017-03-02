use std::env;
use std::path::PathBuf;

fn main()
{
	// All of our messages will be in the "msg" directory
	let mut lcm_source_dir : PathBuf = env!("CARGO_MANIFEST_DIR").into();
	lcm_source_dir.pop();
	lcm_source_dir.push("msg");

	// Specify when this script needs to be rerun
	println!("cargo:rerun-if-changed={}", lcm_source_dir.display());

	// Then run the generation
	lcm_gen::LcmGen::new()
	                .add_directory(lcm_source_dir)
	                .run();
}
