use sp1_build::{build_program, BuildArgs};

fn main() {
    // Build the zkVM guest program
    build_program(
        "./methods/guest",
        BuildArgs::default(),
    );
}
