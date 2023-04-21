use std::time::Duration;
// use palette::{Hsv, Rgb};

use derive_more::Display;
use futures::stream::{
    self,
    StreamExt,
};
use indicatif::ProgressState;
use indicatif::ProgressStyle;
use owo_colors::OwoColorize;
use owo_colors::Rgb;
use rand::thread_rng;
use rand::Rng;
use smartstring::alias::String;
use tracing::info;
use tracing::info_span;
use tracing::instrument;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use lazy_static::lazy_static;
use ulid::Ulid;

// lazy_static! {
//     static ref SPAN_CHILD_PREFIX_SYMBOL: String = "↳
// ".italic().to_string().into(); }

macro_rules! italicize {
    ($msg:expr) => {
        $msg.italic().to_string().into()
    };
}

macro_rules! color {
    ($msg:expr, $color:expr) => {
        $msg.color($color).to_string().into()
        // $msg.color($color).to_string().into()
    };
}

lazy_static! {
    static ref RIGHT_ARROW: String = italicize!("↳ ");
    static ref RIGHT_ARROW_SYMBOL: String = color!(&*RIGHT_ARROW, Rgb(0, 0, 0));
}

fn elapsed_subsec(state: &ProgressState, writer: &mut dyn std::fmt::Write) {
    let seconds = state.elapsed().as_secs();
    let sub_seconds = (state.elapsed().as_millis() % 1000) / 100;
    let duration = state.elapsed();
    let elapsed_secs = duration.as_secs_f64();

    let dark_green = Rgb(0, 100, 0);
    let green = Rgb(0, 255, 0);
    let yellow = Rgb(255, 255, 0);
    let red = Rgb(255, 0, 0);

    // let msg = format!(" ({seconds}.{sub_seconds}s)").color(gradient).to_string();
    // let _ = writer.write_str(&msg);

    let elapsed = state.elapsed();

    let medium_msg_start_time = Duration::from_secs(4);
    let long_msg_start_time = Duration::from_secs(8);

    if elapsed > long_msg_start_time {
        let gradient = interpolate_color(
            &yellow,
            &red,
            (elapsed_secs - long_msg_start_time.as_secs_f64()) / 3.0,
        );
        let _ = write!(writer, "{}", format!("({seconds}.{sub_seconds})s").color(gradient));
    } else if elapsed > medium_msg_start_time {
        let gradient = interpolate_color(
            &green,
            &yellow,
            (elapsed_secs - medium_msg_start_time.as_secs_f64()) / 3.0,
        );
        let _ = write!(writer, "{}", format!("{seconds}.{sub_seconds}s").color(gradient).italic());
    } else {
        let gradient = interpolate_color(&dark_green, &green, elapsed_secs / 3.0);
        let _ = write!(writer, "{}", format!("{seconds}.{sub_seconds}s").color(gradient).italic());
    }
}

#[instrument]
async fn build_sub_unit(sub_unit: u64) {
    let sleep_time =
        thread_rng().gen_range(Duration::from_millis(5000)..Duration::from_millis(10000));
    tokio::time::sleep(sleep_time).await;

    if thread_rng().gen_bool(0.9) {
        // if thread_rng().gen_bool(0.2) {
        info!("sub_unit did something!");
    }
}

#[derive(Debug, Clone, Copy, Display)]
#[display(fmt = "rudolph")]
pub enum TaskKind {
    Build,
    Run,
    Test,
    Embed,
    Cache,
    Clean,
    DBRead,
}

// NOTE: BEFORE

// pub fn long_running_task_msg() -> String {
//     let mut msg = "Long running task".red().to_string();
//     msg = msg.add("... (".black().to_string().as_str());
//     msg = msg
//         .add("Press Ctrl+C to
// cancel".bright_yellow().italic().to_string().as_str())         .add(")".
// black().to_string().as_str());     msg.into()
// }

// pub fn medium_running_task_msg() -> String {
//     let mut msg = "Medium running task".yellow().to_string();
//     msg = msg.add("... (".black().to_string().as_str());
//     msg = msg
//         .add("Press Ctrl+C to
// cancel".bright_yellow().italic().to_string().as_str())         .add(")".
// black().to_string().as_str());     msg.into()
// }

// NOTE: AFTER

// TODO
// pub fn medium_running_task_msg(task_id: TaskId (Ulid `clamp len to 5`)) ->
// String {

pub fn short_running_task_msg(duration: Duration, task_id: String) -> String {
    let elapsed_secs = duration.as_secs_f64();
    let dark_green = Rgb(0, 100, 0);
    let green = Rgb(0, 255, 0);
    let gradient = interpolate_color(&dark_green, &green, elapsed_secs / 3.0);
    format!(
        "{} {}{}{} {}",
        "Short task".color(gradient).italic(),
        "[".black().italic(),
        task_id.black().italic(),
        "]".black().italic(),
        "...".black(),
    )
    .into()
    // format!("{}{}", "Short task".color(gradient).italic(),
    // "...".black(),).into()
}

pub fn medium_running_task_msg(duration: Duration) -> String {
    let elapsed_secs = duration.as_secs_f64();
    let green = Rgb(0, 255, 0);
    let yellow = Rgb(255, 255, 0);
    let gradient = interpolate_color(&green, &yellow, elapsed_secs / 3.0);
    format!("{}{}", "Medium task".color(gradient).italic(), "...".black(),).into()
}

pub fn long_running_task_msg(duration: Duration) -> String {
    let elapsed_secs = duration.as_secs_f64();
    let yellow = Rgb(255, 255, 0);
    let red = Rgb(255, 0, 0);
    let gradient = interpolate_color(&yellow, &red, elapsed_secs / 6.0);
    format!("{}{}", "Long task".color(gradient).italic(), "...".black(),).into()
}

fn interpolate_color(from: &Rgb, to: &Rgb, t: f64) -> Rgb {
    let r = interpolate(from.0, to.0, t);
    let g = interpolate(from.1, to.1, t);
    let b = interpolate(from.2, to.2, t);
    Rgb(r, g, b)
}

// fn interpolate_color(from: &[u8; 3], to: &[u8; 3], t: f64) -> [u8; 3] {
//     [interpolate(from[0], to[0], t), interpolate(from[1], to[1], t),
// interpolate(from[2], to[2], t)] }

fn interpolate(a: u8, b: u8, t: f64) -> u8 {
    let result = a as f64 * (1.0 - t) + b as f64 * t;
    result.round() as u8
}

#[instrument]
async fn build(unit: u64) {
    let sleep_time =
        thread_rng().gen_range(Duration::from_millis(2500)..Duration::from_millis(5000));
    tokio::time::sleep(sleep_time).await;

    let rand_num: f64 = thread_rng().gen();

    if rand_num < 0.1 {
        tokio::join!(build_sub_unit(0), build_sub_unit(1), build_sub_unit(2));
    } else if rand_num < 0.3 {
        tokio::join!(build_sub_unit(0), build_sub_unit(1));
    } else {
        build_sub_unit(0).await;
    }
}

// Desired output:
// Command: `build`.
// Jobs: In progress: 17. Finished: 3847. Cache hits: 0%. Time elapsed: 38.8s

#[tokio::main]
async fn main() {
    let indicatif_layer = IndicatifLayer::new()
        .with_progress_style(
            ProgressStyle::with_template(
                r"{color_start}{span_child_prefix}{span_fields} -- {span_name}{wide_msg}{elapsed_subsec}{color_end}",
            )
            .expect("Failed to initialize TUI")
            .with_key("elapsed_subsec", elapsed_subsec)
            .with_key("color_start", |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                let elapsed = state.elapsed();

                let short_msg_start_time = Duration::from_secs(1);
                let medium_msg_start_time = Duration::from_secs(4);
                let long_msg_start_time = Duration::from_secs(8);
                let very_long_msg_start_time = Duration::from_secs(12);

                let task_id = Ulid::new().to_string().chars().take(5).collect::<String>().to_lowercase().to_string();
                let task_id_color: Rgb = color_from(&task_id);
                // let task_id = Ulid::new().to_string().chars().take(5).collect::<String>().to_lowercase().to_string().cyan().bold().to_string();

                if elapsed > long_msg_start_time {
                    // Red
                    let _ = write!(writer, "{} ", long_running_task_msg(elapsed - long_msg_start_time));
                    // let _ = write!(writer, "\x1b[{}m", 1 + 30);
                } else if elapsed > medium_msg_start_time {
                    // Yellow
                    let _ = write!(writer, "{} ", medium_running_task_msg(elapsed - medium_msg_start_time));
                } else if elapsed > Duration::from_secs(1) {
                    // Green
                    let _ = write!(writer, "{} ", short_running_task_msg(elapsed - short_msg_start_time, task_id.into()));
                }
            })
            .with_key(
                "color_end",
                |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                    if state.elapsed() > Duration::from_secs(4) {
                        let _ = write!(writer, "\x1b[0m");
                    }
                },
            ),
        )
        .with_span_child_prefix_symbol(&RIGHT_ARROW_SYMBOL)
        .with_span_child_prefix_indent(" ");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .init();

    let ulid = ulid::Ulid::new();
    // clamp len to 5
    tracing::info!("BUILD ID: {}", ulid.to_string()[..10].to_string().green().italic());

    // Desired output:
    // Command: `build`.
    // Jobs: In progress: 17. Finished: 3847. Cache hits: 0%. Time elapsed: 38.8s

    // phases:
    // - scheduling
    //   - lays out an acyclic task graph
    // - executing
    //   - runs the task graph
    //     - runs tasks concurrently using a thread pool of configurable size
    //       (default: number of logical cores)
    //     - tasks

    let template = "Executing tasks for command: `build`. {wide_msg} Jobs: In progress: 7. \
                    Finished: 4799. Cache hits: 0%. Time elapsed: {elapsed_subsec}
\n{wide_bar}";

    let header_span = info_span!("header");
    header_span.pb_set_style(
        &ProgressStyle::with_template(template)
            .unwrap()
            .with_key("elapsed_subsec", elapsed_subsec)
            .progress_chars("---"),
    );
    header_span.pb_start();

    // Bit of a hack to show a full "-----" line underneath the header.
    header_span.pb_set_length(1);
    header_span.pb_set_position(1);

    stream::iter((0..20).map(build)).buffer_unordered(7).collect::<Vec<()>>().await;
}

fn color_from(s: &str) -> Rgb {
    // Map each character in the string to an HSV value
    // let hues = s.chars().map(|c| c as f32 / 255.0);
    // let hsv = Hsv::new(hues.fold(0.0, |a, b| a + b), 1.0, 1.0);
    // // Convert the HSV value to an RGB value
    // Rgb::from_hsv(hsv)
    // TODO
    Rgb(0, 0, 0)
}
