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

// ----------------------------

// use clap::{Args, Parser};

// #[derive(Parser)]
// #[command(author = "pulanski", version = "0.1.0")]
// struct Cli {
//     #[command(flatten)]
//     vers: Vers,

//     /// some regular input
//     #[arg(group = "input")]
//     input_file: Option<String>,

//     /// some special input argument
//     #[arg(long, group = "input")]
//     spec_in: Option<String>,

//     #[arg(short, requires = "input")]
//     config: Option<String>,
// }

// #[derive(Args)]
// #[group(required = true, multiple = false)]
// struct Vers {
//     /// set version manually
//     #[arg(long, value_name = "VER")]
//     set_ver: Option<String>,

//     /// auto inc major
//     #[arg(long)]
//     major: bool,

//     /// auto inc minor
//     #[arg(long)]
//     minor: bool,

//     /// auto inc patch
//     #[arg(long)]
//     patch: bool,
// }

// fn main() {
//     let cli = Cli::parse();

//     // Let's assume the old version 1.2.3
//     let mut major = 1;
//     let mut minor = 2;
//     let mut patch = 3;

//     // See if --set_ver was used to set the version manually
//     let vers = &cli.vers;
//     let version = if let Some(ver) = vers.set_ver.as_deref() {
//         ver.to_string()
//     } else {
//         // Increment the one requested (in a real program, we'd reset the lower numbers)
//         let (maj, min, pat) = (vers.major, vers.minor, vers.patch);
//         match (maj, min, pat) {
//             (true, _, _) => major += 1,
//             (_, true, _) => minor += 1,
//             (_, _, true) => patch += 1,
//             _ => unreachable!(),
//         };
//         format!("{major}.{minor}.{patch}")
//     };

//     println!("Version: {version}");

//     // Check for usage of -c
//     if let Some(config) = cli.config.as_deref() {
//         let input = cli
//             .input_file
//             .as_deref()
//             .unwrap_or_else(|| cli.spec_in.as_deref().unwrap());
//         println!("Doing work using input {input} and config {config}");
//     }
// }
