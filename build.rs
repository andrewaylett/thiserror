use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::str;

// This code exercises the surface area that we expect of the Provider API. If
// the current toolchain is able to compile it, then thiserror is able to use
// providers for backtrace support.
const PROVIDE_ANY_PROBE: &str = r#"
    #![feature(provide_any)]

    use std::any::{Demand, Provider};

    fn _f<'a, P: Provider>(p: &'a P, demand: &mut Demand<'a>) {
        p.provide(demand);
    }
"#;

// This code checks to see if std::backtrace::Backtrace is available. If the
// current toolchain is able to compile it, then thiserror should run tests
// relating to backtrace support
const BACKTRACE_PROBE: &str = r#"
    use std::backtrace::Backtrace;

    fn _f() -> Backtrace {
        Backtrace::capture()
    }
"#;

fn main() {
    validate_probe(PROVIDE_ANY_PROBE, "provide_any");
    validate_probe(BACKTRACE_PROBE, "has_backtrace");
}

fn validate_probe(probe: &str, config_name: &str) {
    match compile_probe(probe) {
        Some(status) if status.success() => println!("cargo:rustc-cfg={}", config_name),
        _ => {}
    }
}

fn compile_probe(probe: &str) -> Option<ExitStatus> {
    let rustc = env::var_os("RUSTC")?;
    let out_dir = env::var_os("OUT_DIR")?;
    let probefile = Path::new(&out_dir).join("probe.rs");
    fs::write(&probefile, probe).ok()?;

    // Make sure to pick up Cargo rustc configuration.
    let mut cmd = if let Some(wrapper) = env::var_os("RUSTC_WRAPPER") {
        let mut cmd = Command::new(wrapper);
        // The wrapper's first argument is supposed to be the path to rustc.
        cmd.arg(rustc);
        cmd
    } else {
        Command::new(rustc)
    };

    cmd.stderr(Stdio::null())
        .arg("--edition=2018")
        .arg("--crate-name=thiserror_build")
        .arg("--crate-type=lib")
        .arg("--emit=metadata")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(probefile);

    if let Some(target) = env::var_os("TARGET") {
        cmd.arg("--target").arg(target);
    }

    // If Cargo wants to set RUSTFLAGS, use that.
    if let Ok(rustflags) = env::var("CARGO_ENCODED_RUSTFLAGS") {
        if !rustflags.is_empty() {
            for arg in rustflags.split('\x1f') {
                cmd.arg(arg);
            }
        }
    }

    cmd.status().ok()
}
