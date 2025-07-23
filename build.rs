use std::process::Command;

fn main() {
	println!("cargo::rerun-if-changed=src/shaders/shader.vert");
	let vert_output = Command::new("glslc")
		.args(["src/shaders/shader.vert", "-o", "src/shaders/shader.vert.spv"])
		.output().unwrap();
	if !vert_output.status.success() {
		panic!("{}", str::from_utf8(&vert_output.stderr).unwrap());
	}
	
	println!("cargo::rerun-if-changed=src/shaders/shader.frag");
	let frag_output = Command::new("glslc")
		.args(["src/shaders/shader.frag", "-o", "src/shaders/shader.frag.spv"])
		.output().unwrap();
	if !frag_output.status.success() {
		panic!("{}", str::from_utf8(&frag_output.stderr).unwrap());
	}
}