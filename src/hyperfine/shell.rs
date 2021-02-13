use std;
use std::io;
use std::process::{Command, ExitStatus, Stdio};

use crate::hyperfine::timer::get_cpu_timer;

/// Used to indicate the result of running a command
#[derive(Debug, Copy, Clone)]
pub struct ExecuteResult {
    /// The amount of user time the process used
    pub user_time: f64,

    /// The amount of cpu time the process used
    pub system_time: f64,

    /// The exit status of the process
    pub status: ExitStatus,
}

/// Execute the given command and return a timing summary
#[cfg(windows)]
pub fn execute_and_time(
    stdout: Stdio,
    stderr: Stdio,
    command: &str,
    shell: &str,
) -> io::Result<ExecuteResult> {
    let mut child = run_shell_command(stdout, stderr, command, shell)?;
    let cpu_timer = get_cpu_timer(&child);
    let status = child.wait()?;

    let (user_time, system_time) = cpu_timer.stop();
    Ok(ExecuteResult {
        user_time,
        system_time,
        status,
    })
}

/// Execute the given command and return a timing summary
#[cfg(not(windows))]
pub fn execute_and_time(
    stdout: Stdio,
    stderr: Stdio,
    command: &str,
    shell: &str,
) -> io::Result<ExecuteResult> {
    let cpu_timer = get_cpu_timer();

    let status = run_shell_command(stdout, stderr, command, shell)?;

    let (user_time, system_time) = cpu_timer.stop();

    Ok(ExecuteResult {
        user_time,
        system_time,
        status,
    })
}

/// Run a standard shell command using `sh -c`
#[cfg(not(windows))]
fn run_shell_command(
    stdout: Stdio,
    stderr: Stdio,
    command: &str,
    shell: &str,
) -> io::Result<std::process::ExitStatus> {
    let (executable, arguments) = if let Some(command) = prepare_shell_command(command, shell, "-c")? {
        command
    } else {
        return Ok (std::os::unix::process::ExitStatusExt::from_raw(0));
    };
    Command::new(executable)
        .args(arguments)
        .env(
            "HYPERFINE_RANDOMIZED_ENVIRONMENT_OFFSET",
            "X".repeat(rand::random::<usize>() % 4096usize),
        )
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .status()
}

/// Run a Windows shell command using `cmd.exe /C`
#[cfg(windows)]
fn run_shell_command(
    stdout: Stdio,
    stderr: Stdio,
    command: &str,
    shell: &str,
) -> io::Result<std::process::Child> {
    let (executable, arguments) = if let Some(command) = prepare_shell_command(command, shell, "/C")? {
        command
    } else {
        return Ok (std::os::windows::process::ExitStatusExt::from_raw(0));
    };
    Command::new(executable)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
}

fn prepare_shell_command(
    command: &str,
    shell: &str,
    shell_arg: &str,
) -> io::Result<Option<(String, Vec<String>)>> {
    if shell == "" {
        let mut tokens = match shell_words::split(command) {
            Ok(tokens) => tokens.into_iter(),
            Err(error) => return Err(io::Error::new(io::ErrorKind::Other, format!("{}", error))),
        };
        if let Some(token) = tokens.next() {
            Ok(Some((
                    String::from(token),
                    tokens.map(String::from).collect(),
                )))
        } else {
            Ok(None)
        }
    } else {
        Ok(Some((
                String::from(shell),
                vec![String::from(shell_arg), String::from(command)],
            )))
    }
}

