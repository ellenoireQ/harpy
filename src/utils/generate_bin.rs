use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct GenBin {
    pub input: PathBuf,
    pub output: PathBuf,
}

impl GenBin {
    pub fn build(self) {
        if !self.input.exists() {
            eprintln!("error: missing path input: {:?}", self.input);
            return;
        }

        let result = Command::new("rustc")
            .arg(self.input.clone())
            .arg("-o")
            .arg(&self.output)
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("\n{}", stdout);
                    self.run();
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("\n{}", stderr);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    pub fn run(self) {
        if !self.output.exists() {
            eprintln!("error: missing path output: {:?}", self.output);
            return;
        }

        let run_path = PathBuf::from(format!("./{}", self.output.display()));

        let result = Command::new(run_path).output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("\n{}", stdout);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("\n{}", stderr);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
}

pub fn generate_cargo_package(generated_file: &str) -> std::io::Result<()> {
    let package_dir = PathBuf::from("bindings");
    let src_dir = package_dir.join("src");
    let cargo_toml_path = package_dir.join("Cargo.toml");

    if !package_dir.exists() {
        fs::create_dir_all(&src_dir)?;

        let cargo_toml = r#"[package]
name = "bindings"
version = "0.1.0"
edition = "2024"

[dependencies]
"#;

        fs::write(cargo_toml_path, cargo_toml)?;
    } else {
        fs::create_dir_all(&src_dir)?;
    }

    let generated_code = fs::read_to_string(generated_file)?;
    fs::write(src_dir.join("main.rs"), generated_code)?;

    Ok(())
}
