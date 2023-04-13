fn main() {
    let path = "/home/user/Documents/file.txt";

    // Define ANSI escape codes for different colors
    const RESET_COLOR: &str = "\u{001B}[0m"; // Reset color to default
    const DIR_COLOR: &str = "\u{001B}[34m"; // Blue color for directories
    const FILE_COLOR: &str = "\u{001B}[32m"; // Green color for files
    const SLASH_COLOR: &str = "\u{001B}[36m"; // Cyan color for slashes
    const MAGENTA_COLOR: &str = "\u{001B}[35m"; // Magenta color
    const YELLOW_COLOR: &str = "\u{001B}[33m"; // Yellow color

    // Split the path into directories and file
    let components: Vec<&str> = path.split('/').collect();

    // Iterate over the components and apply color based on type
    for (i, component) in components.iter().enumerate() {
        if component.is_empty() {
            // Skip empty components
            continue;
        }

        if i == components.len() - 1 {
            // Last component is a file
            print!("{}{}{}", FILE_COLOR, component, RESET_COLOR);
        } else {
            // Non-last component is a directory
            print!("{}{}{}", DIR_COLOR, component, RESET_COLOR);
        }

        if i < components.len() - 1 {
            // Add colored slash after directory components
            print!("{}{}{}", SLASH_COLOR, "/", RESET_COLOR);
        }
    }

    // Print additional example paths with different color code combinations
    println!("\nExample paths with different color codes:");
    println!(
        "{}usr{}/{}{}local{}/{}{}bin{}/{}{}script.sh{}",
        DIR_COLOR,
        SLASH_COLOR,
        RESET_COLOR,
        MAGENTA_COLOR,
        SLASH_COLOR,
        RESET_COLOR,
        FILE_COLOR,
        SLASH_COLOR,
        RESET_COLOR,
        YELLOW_COLOR,
        RESET_COLOR,
    );
}
