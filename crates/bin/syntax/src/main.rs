use rand::{Rng, SeedableRng};

fn main() {
    // generate random path of given length
    let random_path = |length: usize| {
        let mut rng = rand::thread_rng();
        let mut path = String::new();
        for _ in 0..length {
            let dir_len = rng.gen_range(0..10);

            // generate random directory name given length
            let dir: String = (0..dir_len)
                .map(|_| rng.gen_range(b'a'..=b'z') as char)
                .collect();

            path.push_str(&format!("{dir}/"));
        }
        path
    };

    let path = random_path(10);

    // Define ANSI escape codes for different colors
    const RESET_COLOR: &str = "\u{001B}[0m"; // Reset color to default

    // Define a vector of color codes for each directory depth
    let mut color_sequence: Vec<String> = vec![
        String::from("\u{001B}[34m"), // Blue for depth 0 (root)
        String::from("\u{001B}[35m"), // Magenta for depth 1
        String::from("\u{001B}[32m"), // Green for depth 2
        String::from("\u{001B}[33m"), // Yellow for depth 3
        String::from("\u{001B}[36m"), // Cyan for depth 4
    ];

    // seed
    // let seed: u8 = 255;
    // let mut rng = rand::rngs::StdRng::seed_from_u64(seed as u64);

    // Generate random color codes for any additional depths
    let mut rng = rand::thread_rng();

    while color_sequence.len() < 10 {
        let color_num = rng.gen_range(16..256);
        println!("color_num: {color_num}");
        let color = format!("\u{001B}[38;5;{}m", rng.gen_range(16..256)); // Generate random 8-bit color
        if !color_sequence.contains(&color) {
            color_sequence.push(color); // Push a clone of the string onto the vector
        }
    }

    // Split the path into directories and file
    let components: Vec<&str> = path.split('/').collect();

    // Iterate over the components and apply color based on depth
    for (i, component) in components.iter().enumerate() {
        if component.is_empty() {
            // Skip empty components
            continue;
        }

        let depth = i;
        let reset_color = RESET_COLOR.to_string();
        let color = color_sequence
            .get(depth % color_sequence.len())
            .unwrap_or(&reset_color);

        print!("{color}{component}{RESET_COLOR}");

        if i < components.len() - 1 {
            // Add colored slash after directory components
            print!(
                "{}/{}",
                color_sequence[depth % color_sequence.len()],
                RESET_COLOR
            );
        }
    }
}
