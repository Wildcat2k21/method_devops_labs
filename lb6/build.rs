//! Генерация Rust-кода gRPC из `proto/pharmacy.proto`.
//!
//! Вместо внешнего `protoc` используется `protox` — компилятор `.proto` на
//! чистом Rust. Поэтому сборка не требует ничего, кроме cargo, и одинаково
//! работает локально, в GitHub Actions и в Docker-образе на Alpine.

use std::env;
use std::fs;
use std::path::PathBuf;

use prost::Message;

const PROTO: &str = "proto/pharmacy.proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={PROTO}");

    let fds = protox::compile([PROTO], ["proto"])?;

    // Дескриптор нужен серверу gRPC reflection (его использует grpcurl).
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    fs::write(out_dir.join("pharmacy_descriptor.bin"), fds.encode_to_vec())?;

    tonic_prost_build::configure().compile_fds(fds)?;
    Ok(())
}
