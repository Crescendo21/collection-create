use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};

fn compress(binary: Vec<u8>) -> Result<Vec<u8>> {
    let mut writer = GzEncoder::new(Vec::<u8>::with_capacity(binary.len()), Compression::best());
    writer.write_all(&binary)?;
    Ok(writer.finish()?)
}

fn main() {
    // Évite la récursion du build script
    if std::env::var("BUILD_IN_PROGRESS").is_ok() {
        println!("Build script already running, skipping to prevent recursion");
        return;
    }
   
    std::env::set_var("BUILD_IN_PROGRESS", "1");
    
    println!("🐕 Starting Giga Dogi build process...");
    
    // Utilise le répertoire courant du projet (plus simple et sûr)
    let current_dir = env::current_dir().expect("Failed to get current directory");
    println!("📁 Current directory: {:?}", current_dir);
    
    // Vérifie que nous sommes bien dans un projet Rust
    let cargo_toml_path = current_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        panic!("Cargo.toml not found in current directory: {:?}", current_dir);
    }
    
    // Créé le répertoire de sortie pour les WASM
    let target_dir = current_dir.join("target");
    let wasm_target_dir = target_dir.join("wasm32-unknown-unknown").join("release");
    
    println!("📦 Building WASM target...");
    
    // Build pour WASM directement dans le répertoire courant
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to execute cargo build");

    if !status.success() {
        panic!("WASM build failed");
    }
    
    println!("✅ WASM build completed successfully!");
    
    // Nom du fichier WASM généré
    let mod_name = "giga_dogi";
    let wasm_file_path = wasm_target_dir.join(format!("{}.wasm", mod_name));
    
    // Vérifie que le fichier WASM existe
    if !wasm_file_path.exists() {
        println!("⚠️  WASM file not found at: {:?}", wasm_file_path);
        println!("🔍 This is normal for library crates. Skipping compression.");
        return;
    }
    
    println!("📁 WASM file found: {:?}", wasm_file_path);
    
    // Lit et compresse le fichier WASM
    match fs::read(&wasm_file_path) {
        Ok(wasm_bytes) => {
            match compress(wasm_bytes.clone()) {
                Ok(compressed) => {
                    let compressed_path = wasm_target_dir.join(format!("{}.wasm.gz", mod_name));
                    
                    if let Err(e) = fs::write(&compressed_path, &compressed) {
                        println!("⚠️  Failed to write compressed file: {}", e);
                    } else {
                        println!("🐕 Giga Dogi WASM compiled and compressed successfully!");
                        println!("📁 WASM Location: {:?}", wasm_file_path);
                        println!("📦 Compressed: {:?}", compressed_path);
                        println!("📊 Original size: {} bytes", wasm_bytes.len());
                        println!("📊 Compressed size: {} bytes", compressed.len());
                        println!("💫 Compression ratio: {:.1}%", (compressed.len() as f64 / wasm_bytes.len() as f64) * 100.0);
                    }
                },
                Err(e) => println!("⚠️  Failed to compress WASM: {}", e)
            }
        },
        Err(e) => println!("⚠️  Failed to read WASM file: {}", e)
    }
}