use std::io::{
    self,
    Write,
};
use std::thread;
use std::time::Duration;

fn main() {
    let width = 50;
    for i in 0..=width {
        let progress = i as f32 / width as f32;
        let color = get_color(progress);
        print!("{}[{}m", 27 as char, color);
        print!("{:width$} [{}%]\r", "", (progress * 100.0) as i32, width = width);
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(100));
    }
}

fn get_color(progress: f32) -> u32 {
    let r = if progress < 0.5 { 255 as f32 } else { (1.0 - progress) * 2.0 * 255.0 };
    let g = 255;
    let b = 0;
    16 + (r as u32) * 256 * 256 + (g as u32) * 256 + b as u32
}
