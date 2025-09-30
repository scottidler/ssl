#![cfg_attr(debug_assertions, allow(unused_imports, unused_variables, unused_mut, dead_code))]

use clap::{Parser, Subcommand};
use eyre::{eyre, Result};
use regex::Regex;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[clap(author, version = env!("GIT_DESCRIBE"), about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Inspect {
        #[clap(value_parser)]
        domain: String,
    },
    Sans {
        #[clap(value_parser)]
        domain: String,
    },
    Validity {
        #[clap(value_parser)]
        domain: String,
    },
    Compare {
        #[clap(value_parser)]
        domain1: String,
        #[clap(value_parser)]
        domain2: String,
    },
}

#[derive(Debug)]
enum InputType {
    Domain(String),
    File(String),
    Stdin(String),
}

fn is_stdin_empty() -> Result<bool, io::Error> {
    let mut buffer = [0; 1];
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    match handle.read(&mut buffer) {
        Ok(0) | Err(_) => Ok(true),
        Ok(_) => Ok(false),
    }
}

fn input_type(input: &str) -> Result<InputType> {
    if Path::new(input).exists() && fs::metadata(input)?.is_file() {
        Ok(InputType::File(input.to_string()))
    } else if input.contains('.') && !input.contains('/') {
        Ok(InputType::Domain(input.to_string()))
    } else if !is_stdin_empty()? {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(InputType::Stdin(buffer))
    } else {
        Err(eyre!("Input does not match any expected type"))
    }
}

fn fetch_certificate_from_domain(domain: &str) -> Result<String> {
    let mut cmd = Command::new("openssl");
    cmd.args(&[
        "s_client",
        "-connect",
        &format!("{}:443", domain),
        "-servername",
        domain,
        //"-showcerts",
    ]);
    execute_command(cmd, None)
}

fn execute_command(mut cmd: Command, input_data: Option<&str>) -> Result<String> {
    println!("execute_command: cmd: {:?}", cmd);
    if let Some(data) = input_data {
        cmd.stdin(Stdio::piped());
    }
    cmd.stderr(Stdio::piped());

    let output = if let Some(data) = input_data {
        //println!("execute_command: input_data: {}", data);
        let mut child = cmd.spawn()?;
        if let Some(ref mut stdin) = child.stdin.take() {
            stdin.write_all(data.as_bytes())?;
        } else {
            return Err(eyre!("Failed to open stdin"));
        }
        child.wait_with_output()?
    } else {
        println!("execute_command: no input_data");
        cmd.output()?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!(
            "Command execution failed with status: {:?}, stderr: {}",
            output.status,
            stderr
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if stdout.is_empty() && stderr.is_empty() {
        return Err(eyre!("Both stdout and stderr are empty"));
    }

    println!("execute_command: stdout: {}", stdout);
    println!("execute_command: stderr: {}", stderr);

    Ok(stdout)
}

fn inspect(input: &str) -> Result<String> {
    let result = match input_type(input)? {
        InputType::Domain(domain) => {
            let certificate_data = fetch_certificate_from_domain(&domain)?;
            let mut x509_cmd = Command::new("openssl");
            x509_cmd
                .args(&["x509", "-noout", "-text"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped());
            execute_command(x509_cmd, Some(&certificate_data))
        }
        InputType::File(file_path) => {
            let mut x509_cmd = Command::new("openssl");
            x509_cmd
                .args(&["x509", "-in", &file_path, "-text", "-noout"])
                .stdout(Stdio::piped());
            execute_command(x509_cmd, None)
        }
        InputType::Stdin(stdin_content) => {
            //let x509_cmd = create_x509_command();
            let mut x509_cmd = Command::new("openssl");
            x509_cmd
                .args(&["x509", "-noout", "-text"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped());
            execute_command(x509_cmd, Some(&stdin_content))
        }
    };
    println!("inspect: result: {:?}", result);
    result
}

fn sans(domain: &str) -> Result<String> {
    let inspect_output = inspect(domain)?;
    println!("Full output from inspect:\n{}", inspect_output);

    let mut count = 0;
    let lines: Vec<&str> = inspect_output.split('\n').collect();
    for line in lines.iter() {
        count += 1;
        println!("{}: {}", count, line);
    }

    Ok(inspect_output)
}

fn validity(domain: &str) -> Result<String> {
    let inspect_output = inspect(domain)?;
    // Extract validity information from the inspect_output
    // Similar to the Sans function, parse the output to find the validity dates
    // Return the validity information or an error if something goes wrong
    todo!()
}

fn compare(domain1: &str, domain2: &str) -> Result<String> {
    let inspect_output1 = inspect(domain1)?;
    let inspect_output2 = inspect(domain2)?;
    // Compare the outputs
    // You can decide how to compare (e.g., direct string comparison, structured parsing, etc.)
    // Return a message indicating whether they match or not
    todo!()
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Inspect { domain } => match inspect(&domain) {
            Ok(result) => println!("{}", result),
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Sans { domain } => match sans(&domain) {
            Ok(result) => println!("{}", result),
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Validity { domain } => match validity(&domain) {
            Ok(result) => println!("{}", result),
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Compare { domain1, domain2 } => match compare(&domain1, &domain2) {
            Ok(result) => println!("{}", result),
            Err(e) => eprintln!("Error: {}", e),
        },
    }
}
