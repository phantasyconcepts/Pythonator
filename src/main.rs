use std::fs;
use std::io::{self, Write};
use std::process::Command;
use std::path::Path;

/// Configuration for the Pythonator Generation Pipeline
#[derive(Debug, Clone)]
pub struct PythonatorConfig {
    pub max_iterations: usize,
    pub sandbox_target_dir: String,
    pub optimization_level: u8,
}

/// Represents the state of a code generation task
#[derive(Debug)]
pub struct GenerationTask {
    pub task_id: String,
    pub prompt: String,
    pub generated_code: Option<String>,
    pub iteration_count: usize,
    pub last_error: Option<String>,
}

impl GenerationTask {
    pub fn new(task_id: &str, prompt: &str) -> Self {
        Self {
            task_id: task_id.to_string(),
            prompt: prompt.to_string(),
            generated_code: None,
            iteration_count: 0,
            last_error: None,
        }
    }
}

/// The Core Orchestrator Agent for Pythonator
pub struct PythonatorOrchestrator {
    config: PythonatorConfig,
}

impl PythonatorOrchestrator {
    pub fn new(config: PythonatorConfig) -> Self {
        // Ensure sandbox environment directory exists
        if !Path::new(&config.sandbox_target_dir).exists() {
            fs::create_dir_all(&config.sandbox_target_dir).expect("Failed to create sandbox directory");
        }
        Self { config }
    }

    /// Primary execution pipeline mapping prompt to verified code output
    pub fn execute_pipeline(&self, task: &mut GenerationTask) -> Result<String, String> {
        println!("[Orchestrator] Starting synthesis pipeline for task: {}", task.task_id);

        while task.iteration_count < self.config.max_iterations {
            task.iteration_count += 1;
            println!("\n[Pipeline] Iteration {}/{}", task.iteration_count, self.config.max_iterations);

            // Step 1: Code Synthesis / Refinement
            let candidate_code = self.synthesize_code(task);
            task.generated_code = Some(candidate_code.clone());

            // Step 2: Sandboxed Verification Environment Test
            match self.verify_in_sandbox(&candidate_code, &task.task_id) {
                Ok(()) => {
                    println!("[Sandbox] Verification successful. Code passed syntax and structural checks.");
                    return Ok(candidate_code);
                }
                Err(err_msg) => {
                    println!("[Sandbox] Verification failed: {}", err_msg);
                    task.last_error = Some(err_msg);
                    // Loop continues with error feedback for self-correction
                }
            }
        }

        Err("Maximum iteration threshold reached without achieving a clean sandbox build.".to_string())
    }

    /// Simulates code synthesis leveraging previous error feedback if available
    fn synthesize_code(&self, task: &GenerationTask) -> String {
        println!("[Synthesizer] Curating code components based on prompt: '{}'", task.prompt);
        
        if let Some(ref err) = task.last_error {
            println!("[Synthesizer] Self-correcting based on previous compiler/runtime error: {}", err);
            // In a full production setup, error context is fed into the LLM/generator prompt backend here.
        }

        // Example template output matching the Pythonator structural target
        format!(
            r#""# 
            r#""
            # Auto-generated Pythonator Module: {}
            # Generated via Rust Pythonator Engine

            import sys

            def run_module():
                print("Initializing Pythonator Target System...")
                # Core logic implementation matching prompt specs
                return True

            if __name__ == "__main__":
                success = run_module()
                sys.exit(0 if success else 1)
            "#,
            task.task_id
        )
    }

    /// Isolates execution inside a local sandboxed directory and checks validity via Python interpreter
    fn verify_in_sandbox(&self, code: &str, task_id: &str) -> Result<(), String> {
        let file_name = format!("{}/{}.py", self.config.sandbox_target_dir, task_id);
        
        // Write generated code to sandbox file
        let mut file = fs::File::create(&file_name).map_err(|e| e.to_string())?;
        file.write_all(code.as_bytes()).map_err(|e| e.to_string())?;

        println!("[Sandbox] Executing static analysis and syntax compilation check...");

        // Invoke python interpreter in syntax check mode (-m py_compile) for secure sandboxed validation
        let output = Command::new("python3")
            .arg("-m")
            .arg("py_compile")
            .arg(&file_name)
            .output();

        match output {
            Ok(res) => {
                if res.status.success() {
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&res.stderr).into_owned();
                    Err(stderr)
                }
            }
            Err(e) => Err(format!("Failed to spawn sandbox compiler process: {}", e)),
        }
    }
}

fn main() {
    println!("=== Pythonator Rust Meta-Programming Engine ===");

    let config = PythonatorConfig {
        max_iterations: 3,
        sandbox_target_dir: "./sandbox_target".to_string(),
        optimization_level: 2,
    };

    let orchestrator = PythonatorOrchestrator::new(config);
    let mut task = GenerationTask::new(
        "module_core_processor", 
        "Build a robust data handler module with automated exception management."
    );

    match orchestrator.execute_pipeline(&mut task) {
        Ok(final_code) => {
            println!("\n[Success] Pythonator successfully compiled and verified target code:");
            println!("--------------------------------------------------");
            println!("{}", final_code);
            println!("--------------------------------------------------");
        }
        Err(e) => {
            eprintln!("\n[Error] Pipeline execution failed: {}", e);
        }
    }
}
