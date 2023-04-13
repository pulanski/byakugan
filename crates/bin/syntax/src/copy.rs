// fn main() {
//     let path = "/home/user/Documents/file.txt";

//     // Define ANSI escape codes for different colors
//     const RESET_COLOR: &str = "\u{001B}[0m"; // Reset color to default

//     // Define a vector of color codes for each directory depth
//     let mut color_sequence: Vec<&str> = vec![
//         "\u{001B}[34m", // Blue for depth 0 (root)
//         "\u{001B}[35m", // Magenta for depth 1
//         "\u{001B}[32m", // Green for depth 2
//         "\u{001B}[33m", // Yellow for depth 3
//         "\u{001B}[36m", // Cyan for depth 4
//     ];

//     // Generate random color codes for any additional depths
//     let mut rng = rand::thread_rng();
//     while color_sequence.len() < 10 {
//         // Arbitrary limit to prevent infinite loop
//         let color = format!("\u{001B}[38;5;{}m", rng.gen_range(16..256)); // Generate random 8-bit color
//         if !color_sequence.contains(&&color[..]) {
//             color_sequence.push(&color[..]);
//         }
//     }

//     // Split the path into directories and file
//     let components: Vec<&str> = path.split('/').collect();

//     // Iterate over the components and apply color based on depth
//     for (i, component) in components.iter().enumerate() {
//         if component.is_empty() {
//             // Skip empty components
//             continue;
//         }

//         let depth = i as usize;
//         let color = color_sequence
//             .get(depth % color_sequence.len())
//             .unwrap_or(&RESET_COLOR);

//         print!("{}{}{}", color, component, RESET_COLOR);

//         if i < components.len() - 1 {
//             // Add colored slash after directory components
//             print!(
//                 "{}{}{}",
//                 color_sequence[depth % color_sequence.len()],
//                 "/",
//                 RESET_COLOR
//             );
//         }
//     }
// }
