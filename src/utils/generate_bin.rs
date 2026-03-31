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
