use std::io::stdin;
use std::process::Command;
/// Opens files in a new terminal using the specified editor command.
///
/// # Arguments
///
/// * `default_editor_command` - The command to open the editor.
/// * `file_number` - The file number to open.
/// * `potential_hits` - A vector of tuples containing file information (index, name, full path).
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - Returns Ok(()) if successful, otherwise returns an error.
///
/// ``


/// Opens a command in a new terminal window.
///
/// # Arguments
///
/// * `command` - The command to run in the new terminal.
/// * `args` - The arguments to pass to the command.
///
/// # Returns
///
/// * `Result<(), std::io::Error>` - Returns Ok(()) if successful, otherwise returns an error.
///
/// # Platform-specific behavior
///
/// * On Windows, uses `cmd` with `/c start`.
/// * On Linux, uses `gnome-terminal` with `--`.
/// * On macOS, uses `open` with `-a Terminal`.
/// Opens a command in a new terminal window.
pub fn open_file_in_terminal(command: &str, file_path: &str) -> Result<(), std::io::Error> {
    #[cfg(target_os = "windows")]
    let terminal_cmd = "cmd";
    #[cfg(target_os = "windows")]
    let terminal_args = &["/c", "start", command];

    #[cfg(target_os = "linux")]
    let terminal_cmd = "gnome-terminal";
    #[cfg(target_os = "linux")]
    let terminal_args = &["--", command];

    #[cfg(target_os = "macos")]
    let terminal_cmd = "open";
    #[cfg(target_os = "macos")]
    let terminal_args = &["-a", "Terminal", command];

    let mut cmd = Command::new(terminal_cmd);
    cmd.args(terminal_args);
    cmd.arg(file_path);
    cmd.spawn()?;
    Ok(())
}
