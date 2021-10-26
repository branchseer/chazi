#![doc = include_str!("../README.md")]

use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::atomic::Ordering;
use std::time::Duration;

pub use ::chazi_macros::test;

use crate::probe::{generate_probe, parse_probe};
use crate::reached::{CHECK_REACH, parse_reach_info, ReachLineInfo};

mod probe;
pub mod reached;

const CHILD_MARK_ENV_NAME: &str = "CHAZI_CHILD_353887F6_A130_11EB_AAD1_54B203047EBD";

const PANIC_RAISED_MESSAGE: &str = "p";

fn get_test_exact_name(module_path: &str, test_name: &str) -> String {
    if let Some(first_sep_pos) = module_path.find("::") {
        format!("{}::{}", &module_path[(first_sep_pos + 2)..], test_name)
    } else {
        test_name.to_string()
    }
}

#[allow(missing_docs)]
#[doc(hidden)]
pub struct TestConfig {
    pub ignore: bool,
    pub timeout: Duration,
    pub check_reach: bool,
    pub expected_result: TestResult,
    pub parent_should_panic: bool,
}

#[allow(missing_docs)]
#[doc(hidden)]
pub enum TestResult {
    Panic,
    ExitCode(i32),
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            ignore: false,
            timeout: Duration::from_secs(5),
            check_reach: false,
            expected_result: TestResult::ExitCode(0),
            parent_should_panic: false,
        }
    }
}

#[allow(missing_docs)]
#[doc(hidden)]
pub fn fork_in_test(test_fn_module_path: &str, test_name: &str, impl_fn: fn(), config: TestConfig) {
    if std::env::var(CHILD_MARK_ENV_NAME).is_ok() {
        if config.check_reach {
            CHECK_REACH.store(true, Ordering::Relaxed);
        }
        match config.expected_result {
            TestResult::Panic => {
                match std::panic::catch_unwind(impl_fn) {
                    Ok(()) => {}
                    Err(_err) => {
                        // TODO: print out the error
                        eprintln!("{}", generate_probe(PANIC_RAISED_MESSAGE))
                    }
                }
            }
            _ => impl_fn(),
        }
        // Quitting the process prematurely to prevent libtest from outputting "test result: ok. ....."
        std::process::exit(0);
    } else {
        if config.parent_should_panic {
            std::panic::catch_unwind(|| parent(test_fn_module_path, test_name, config))
                .unwrap_err();
        } else {
            parent(test_fn_module_path, test_name, config)
        }
    }
}

fn parent(test_fn_module_path: &str, test_name: &str, config: TestConfig) {
    let test_exact_name = get_test_exact_name(test_fn_module_path, test_name);

    let mut command = Command::new(
        std::env::current_exe().expect("Cannot get the path of the current executable"),
    );
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env(CHILD_MARK_ENV_NAME, "1")
        .args(&[
            test_exact_name.as_str(),
            "--exact",
            "--test-threads=1",
            "--nocapture",
        ]);
    if config.ignore {
        command.arg("--ignored");
    }
    let mut child = command.spawn().expect("Failed to fork a child");

    validate_test_output_prefix(test_exact_name.as_str(), child.stdout.as_mut().unwrap());
    let stdout_task = std::thread::spawn({
        let stdout = child.stdout.take().unwrap();
        move || {
            for line in BufReader::new(stdout).lines() {
                // Using `println!` macro instead of writing directly to stdout,
                // so that cargo test can capture the content.
                println!("{}", line.unwrap())
            }
        }
    });


    let expects_panic = match config.expected_result {
        TestResult::Panic => true,
        _ => false,
    };
    let stderr_task = std::thread::spawn({
        let stderr = child.stderr.take().unwrap();
        let check_reach = config.check_reach;
        move || {
            let mut reach_lines = Vec::<ReachLineInfo>::new();
            let mut panicked: bool = false;
            for line in BufReader::new(stderr).lines() {
                let line = line.unwrap();
                if let Some((line_prefix, probe_content)) = parse_probe(line.as_str()) {
                    if !line_prefix.is_empty() {
                        eprint!("{}", line_prefix)
                    }
                    if expects_panic && probe_content == PANIC_RAISED_MESSAGE {
                        panicked = true;
                        continue;
                    } else if check_reach {
                        if let Some(reach_line) = parse_reach_info(probe_content) {
                            reach_lines.push(reach_line);
                            continue;
                        }
                    }
                    panic!("Unknown probe content: {}", probe_content)
                }
                // Using `eprint!` macro instead of writing directly to stderr,
                // so that cargo test can capture the content.
                eprintln!("{}", line);
            }
            (reach_lines, panicked)
        }
    });

    let exit_status = if config.timeout != Duration::from_nanos(0) {
        wait_timeout::ChildExt::wait_timeout(&mut child, config.timeout)
            .expect("The command wasn't running")
            .expect(format!("Timeout({:?}) exceeded", config.timeout).as_str())
    } else {
        child.wait().expect("The command wasn't running")
    };
    let expected_exit_code = match config.expected_result {
        TestResult::ExitCode(code) => code,
        TestResult::Panic { .. } => 0,
    };
    match exit_status.code() {
        None => panic!("Test process was terminated by a signal"), // According to the doc of ExiStatus.code
        Some(exit_code) => assert_eq!(dbg!(exit_code), dbg!(expected_exit_code)),
    }
    stdout_task.join().unwrap();
    let (reach_lines, panicked) = stderr_task.join().unwrap();
    assert_eq!(expects_panic, panicked);

    if config.check_reach {
        reached::validate_reaches(reach_lines.as_slice());
    }
}

fn validate_test_output_prefix(name: &str, stdout: &mut impl Read) {
    let expected_prefix = format!("\nrunning 1 test\ntest {} ... ", name);

    let mut prefix_buf = vec![0u8; expected_prefix.len()];
    stdout.read_exact(&mut prefix_buf).unwrap();
    assert_eq!(
        std::str::from_utf8(&prefix_buf).unwrap(),
        expected_prefix.as_str()
    );
}
