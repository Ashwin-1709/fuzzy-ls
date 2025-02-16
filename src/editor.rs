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
pub fn experimental_open_files(
    default_editor_command: String,
    file_number: usize,
    potential_hits: Vec<(u32, String, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter file number to open the file in an editor. Press Enter to exit.");
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("Failed to read the input.");

    let index_number: usize = match input.trim().parse() {
        Ok(index) => index,
        Err(_) => return Ok(()),
    };
    if index_number > 0 && index_number <= file_number {
        let (_, _, full_path) = &potential_hits[index_number - 1];
        open_in_new_terminal(&default_editor_command, &[full_path])
            .expect("Failed to open file in the editor.");
    } else {
        println!("Invalid file number.");
    }
    return Ok(());
}

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
fn open_in_new_terminal(command: &str, args: &[&str]) -> Result<(), std::io::Error> {
    #[cfg(target_os = "windows")]
    let terminal_cmd = "cmd";
    #[cfg(target_os = "windows")]
    let terminal_args = &["/c", "start", command];

    #[cfg(target_os = "linux")]
    let terminal_cmd = "gnome-terminal"; // Or "xterm", "konsole", etc. - see below
    #[cfg(target_os = "linux")]
    let terminal_args = &["--", command]; // Important: "--" separates terminal args from command args

    #[cfg(target_os = "macos")]
    let terminal_cmd = "open";
    #[cfg(target_os = "macos")]
    let terminal_args = &["-a", "Terminal", command];

    let mut cmd = Command::new(terminal_cmd);
    cmd.args(terminal_args);
    cmd.args(args); // Add any arguments to your command

    cmd.spawn()?; // Execute the command

    Ok(())
}
