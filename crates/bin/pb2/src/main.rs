use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use cached::proc_macro::cached;
use derive_more::Display;
use derive_new::new;
use futures::stream::{
    self,
    StreamExt,
};
use getset::Getters;
use getset::MutGetters;
use getset::Setters;
use indicatif::ProgressState;
use indicatif::ProgressStyle;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use owo_colors::OwoColorize;
use owo_colors::Rgb;
use rand::thread_rng;
use rand::Rng;
use shrinkwraprs::Shrinkwrap;
use smartstring::alias::String;
use tracing::info;
use tracing::info_span;
use tracing::instrument;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use typed_builder::TypedBuilder;
use ulid::Ulid;

macro_rules! italicize {
    ($msg:expr) => {
        $msg.italic().to_string().into()
    };
}

macro_rules! bold {
    ($msg:expr) => {
        $msg.bold().to_string().into()
    };
}

macro_rules! color {
    ($msg:expr, $color:expr) => {
        $msg.color($color).to_string().into()
    };
}

// Durations for different progress indicators
static SHORT_DURATION_START_TIME: Duration = Duration::from_secs(1);
static MEDIUM_DURATION_START_TIME: Duration = Duration::from_secs(4);
static LONG_DURATION_START_TIME: Duration = Duration::from_secs(8);
static VERY_LONG_DURATION_START_TIME: Duration = Duration::from_secs(12);

// Track in-progress and completed tasks as well as cache hits
static TASK_COUNTER: Lazy<TaskTracker> = Lazy::new(|| TaskTracker::new(0, 0));

lazy_static! {
    // Colors
    static ref RED: Rgb = Rgb(255, 0, 0);
    static ref ORANGE: Rgb = Rgb(255, 165, 0);
    static ref YELLOW: Rgb = Rgb(255, 255, 0);
    static ref GREEN: Rgb = Rgb(0, 255, 0);
    static ref DARK_GREEN: Rgb = Rgb(0, 100, 0);
    static ref BLUE: Rgb = Rgb(0, 0, 255);
    static ref INDIGO: Rgb = Rgb(75, 0, 130);
    static ref VIOLET: Rgb = Rgb(238, 130, 238);
    static ref PURPLE: Rgb = Rgb(255, 0, 255);
    static ref CYAN: Rgb = Rgb(0, 255, 255);
    static ref WHITE: Rgb = Rgb(255, 255, 255);
    static ref BLACK: Rgb = Rgb(0, 0, 0);

    // Symbols
    static ref RIGHT_ARROW: String = italicize!("↳ ");
    static ref RIGHT_ARROW_SYMBOL: String = color!(&*RIGHT_ARROW, Rgb(0, 0, 0));
}

#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    PartialEq,
    Eq,
    Hash,
    TypedBuilder,
    Getters,
    MutGetters,
    Setters,
    new,
)]
#[display(fmt = "In progress {in_progress}, completed {completed}")]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
struct TaskTrackerData {
    in_progress: usize,
    completed:   usize,
}

#[derive(Debug, Clone, Shrinkwrap)]
struct TaskTracker(Arc<RwLock<TaskTrackerData>>);

impl TaskTracker {
    pub fn new(in_progress: usize, completed: usize) -> Self {
        Self(Arc::new(RwLock::new(TaskTrackerData::new(in_progress, completed))))
    }
}

fn elapsed_subsec(state: &ProgressState, writer: &mut dyn std::fmt::Write) {
    let seconds = state.elapsed().as_secs();
    let sub_seconds = (state.elapsed().as_millis() % 1000) / 100;
    let duration = state.elapsed();
    let elapsed_secs = duration.as_secs_f64();

    let elapsed = state.elapsed();

    if elapsed > VERY_LONG_DURATION_START_TIME {
        let gradient = interpolate_color(
            &ORANGE,
            &RED,
            (elapsed_secs - VERY_LONG_DURATION_START_TIME.as_secs_f64()) / 3.0,
        );
        let _ = write!(writer, "{}", format!("{seconds}.{sub_seconds}s").color(gradient));
    } else if elapsed > LONG_DURATION_START_TIME {
        let gradient = interpolate_color(
            &YELLOW,
            &ORANGE,
            (elapsed_secs - LONG_DURATION_START_TIME.as_secs_f64()) / 3.0,
        );
        let _ = write!(writer, "{}", format!("{seconds}.{sub_seconds}s").color(gradient));
    } else if elapsed > MEDIUM_DURATION_START_TIME {
        let gradient = interpolate_color(
            &GREEN,
            &YELLOW,
            (elapsed_secs - MEDIUM_DURATION_START_TIME.as_secs_f64()) / 3.0,
        );
        let _ = write!(writer, "{}", format!("{seconds}.{sub_seconds}s").color(gradient).italic());
    } else {
        let gradient = interpolate_color(&DARK_GREEN, &GREEN, elapsed_secs / 3.0);
        let _ = write!(writer, "{}", format!("{seconds}.{sub_seconds}s").color(gradient).italic());
    }
}

pub fn increment_in_progress_task() {
    let mut task_counter = TASK_COUNTER.write();
    *task_counter.in_progress_mut() += 1;
    tracing::trace!(
        "Started task, in progress: {}, completed: {}",
        task_counter.in_progress(),
        task_counter.completed()
    );
}

pub fn increment_completed_task() {
    let mut task_counter = TASK_COUNTER.write();
    *task_counter.completed_mut() += 1;
    if task_counter.in_progress() > &0 {
        *task_counter.in_progress_mut() -= 1;
    }
    tracing::trace!(
        "Completed task, in progress: {}, completed: {}",
        task_counter.in_progress(),
        task_counter.completed()
    );
}

#[instrument(level = "trace", skip_all)]
#[cached(size = 100)]
async fn build_sub_unit(sub_unit: u64) {
    increment_in_progress_task(); // Increment the in progress task count
    let sleep_time =
        thread_rng().gen_range(Duration::from_millis(5000)..Duration::from_millis(10000));
    tokio::time::sleep(sleep_time).await;

    if thread_rng().gen_bool(0.9) {
        info!("sub_unit {} built successfully", sub_unit);
    }

    increment_completed_task(); // Increment the completed task count
}

pub fn short_running_task_msg(duration: Duration) -> String {
    let elapsed_secs = duration.as_secs_f64();
    let gradient = interpolate_color(&DARK_GREEN, &GREEN, elapsed_secs / 3.0);
    format!("{}{}{}", "[".black(), "Short".color(gradient).italic(), "]".black()).into()
}

pub fn medium_running_task_msg(duration: Duration) -> String {
    let elapsed_secs = duration.as_secs_f64();
    let gradient = interpolate_color(&GREEN, &YELLOW, elapsed_secs / 3.0);
    format!("{}{}{}", "[".black(), "Medium".color(gradient).italic(), "]".black()).into()
}

pub fn long_running_task_msg(duration: Duration) -> String {
    let elapsed_secs = duration.as_secs_f64();
    let gradient = interpolate_color(&YELLOW, &ORANGE, elapsed_secs / 6.0);
    format!("{}{}{}", "[".black(), "Long".color(gradient).italic(), "]".black()).into()
}

pub fn very_long_running_task_msg(duration: Duration) -> String {
    let elapsed_secs = duration.as_secs_f64();
    let gradient = interpolate_color(&ORANGE, &RED, elapsed_secs / 6.0);
    format!("{}{}{}", "[".black(), "Very Long".color(gradient).italic(), "]".black()).into()
}

pub fn cache_hits_msg(percentage: f64) -> String {
    let gradient = interpolate_color(&RED, &GREEN, percentage);
    format!("Cache Hits{} {:.2}", ":".black(), percentage.color(gradient).italic()).into()
}

fn interpolate_color(from: &Rgb, to: &Rgb, t: f64) -> Rgb {
    let r = interpolate(from.0, to.0, t);
    let g = interpolate(from.1, to.1, t);
    let b = interpolate(from.2, to.2, t);
    Rgb(r, g, b)
}

fn interpolate(a: u8, b: u8, t: f64) -> u8 {
    let result = a as f64 * (1.0 - t) + b as f64 * t;
    result.round() as u8
}

#[instrument(level = "trace", skip_all)]
async fn build(unit: u64) {
    increment_in_progress_task(); // Increment in-progress tasks when a new task starts

    // let mut tasks = Vec::new();

    let sleep_time =
        thread_rng().gen_range(Duration::from_millis(2500)..Duration::from_millis(5000));
    tokio::time::sleep(sleep_time).await;

    let rand_num: f64 = thread_rng().gen();

    // if rand_num < 0.1 {
    //     tasks.push(tokio::spawn(build_sub_unit(0)));
    //     tasks.push(tokio::spawn(build_sub_unit(1)));
    //     tasks.push(tokio::spawn(build_sub_unit(2)));
    // } else if rand_num < 0.3 {
    //     tasks.push(tokio::spawn(build_sub_unit(0)));
    //     tasks.push(tokio::spawn(build_sub_unit(1)));
    // } else if rand_num < 0.6 {
    //     tasks.push(tokio::spawn(build_sub_unit(0)));
    // } else {
    //     tasks.push(tokio::spawn(build_sub_unit(0)));
    //     tasks.push(tokio::spawn(build_sub_unit(1)));
    //     tasks.push(tokio::spawn(build_sub_unit(2)));
    //     tasks.push(tokio::spawn(build_sub_unit(3)));
    //     tasks.push(tokio::spawn(build_sub_unit(4)));
    // }

    // for task in tasks {
    //     task.await.expect("Task failed");
    // }
}

#[tokio::main]
async fn main() {
    let start_time = Instant::now();
    let num_units = 10;
    let task = "build";

    let indicatif_layer = IndicatifLayer::new()
        .with_progress_style(
            ProgressStyle::with_template(
                r"{spinner:.green}{color_start}{span_child_prefix}{span_fields} -- {span_name}{wide_msg}{elapsed_subsec}{color_end}",
            )
            .expect("Failed to initialize TUI")
            .tick_strings(&[
            "    ",
            "◐   ",
            "◓   ",
			"=◑  ",
			"==◒ ",
			"===◐",
			" ===",
			"  ==",
			"   =",
			"    ",
			"   ◓",
			"  ◑=",
			" ◒==",
			"◐===",
			"◓== ",
			"◑=  ",
			"◒   ",
            "    ",])
            .with_key("elapsed_subsec", elapsed_subsec)
            .with_key("color_start", |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                task_msg_display(state, writer)
            })
            .with_key(
                "color_end",
                |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                    if state.elapsed() > Duration::from_secs(4) {
                        let _ = write!(writer, "\x1b[0m");
                    }
                },
            )
        )
        .with_span_child_prefix_symbol(&RIGHT_ARROW_SYMBOL)
        .with_span_child_prefix_indent(" ");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(indicatif_layer.get_stderr_writer())
                .fmt_fields(tracing_subscriber::fmt::format::DefaultFields::new())
                .with_line_number(true)
                // .without_time()
                // .with_thread_names(true)
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
                .with_ansi(true)
                .with_timer(tracing_subscriber::fmt::time::Uptime::default()),
        )
        .with(indicatif_layer)
        .init();

    let ulid = Ulid::new();
    let task_id = ulid.to_string()[..10].to_string();
    tracing::info!("{} ID: {}", task, task_id.cyan().italic());

    let task_display = format!(
        "{}{}{} {}{}{}",
        "`".red(),
        task.green(),
        "`".red(),
        "[".black(),
        task_id[..5].to_string().cyan().italic(),
        "]".black()
    );
    // .replace("{task_display}", &task_display)

    let template = "Executing tasks for command: {task_display}. {wide_msg} Jobs: In progress: \
                    {in_progress}. Finished: {completed}. Time elapsed: {elapsed_subsec}
\n{wide_bar}"
        .replace('.', &format!("{}", ".".black()))
        .replace(':', &format!("{}", ":".black()))
        .replace("In progress", &format!("{}", "In progress".bright_yellow()))
        .replace("Finished", &format!("{}", "Finished".green()));

    let header_span = info_span!("header");
    header_span.pb_set_style(
        &ProgressStyle::with_template(&template)
            .unwrap()
            .with_key("elapsed_subsec", elapsed_subsec)
            .with_key("in_progress", |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                let task_counter = TASK_COUNTER.read();

                let _ = write!(writer, "{}", task_counter.in_progress);
            })
            .with_key("completed", |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                let task_counter = TASK_COUNTER.read();

                let _ = write!(writer, "{}", task_counter.completed);
            })
            .progress_chars("---"),
    );
    header_span.pb_start();

    // Display full "-----" line underneath the header.
    header_span.pb_set_length(1);
    header_span.pb_set_position(1);

    stream::iter((0..num_units).map(build)).buffer_unordered(7).collect::<Vec<()>>().await;

    // Display the resulting diagnostics from the task tracker
    let task_counter = TASK_COUNTER.read();

    tracing::info!("Finished executing tasks for command: {}", task_display);
    tracing::info!(
        " Jobs Finished{} {}",
        ":".black(),
        task_counter.completed.green().bold().italic()
    );
    tracing::info!(
        " Time elapsed{} {}{}{}s",
        ":".black(),
        start_time.elapsed().as_secs().to_string().cyan().italic(),
        ".".black(),
        start_time.elapsed().subsec_millis().to_string().cyan().italic()
    );
}

fn task_msg_display(state: &ProgressState, writer: &mut dyn std::fmt::Write) {
    let elapsed = state.elapsed();

    if elapsed > VERY_LONG_DURATION_START_TIME {
        let _ = write!(
            writer,
            " {} ",
            very_long_running_task_msg(elapsed - VERY_LONG_DURATION_START_TIME)
        );
    } else if elapsed > LONG_DURATION_START_TIME {
        let _ = write!(writer, " {} ", long_running_task_msg(elapsed - LONG_DURATION_START_TIME));
    } else if elapsed > MEDIUM_DURATION_START_TIME {
        let _ =
            write!(writer, " {} ", medium_running_task_msg(elapsed - MEDIUM_DURATION_START_TIME));
    } else if elapsed > SHORT_DURATION_START_TIME {
        let _ = write!(writer, " {} ", short_running_task_msg(elapsed - SHORT_DURATION_START_TIME));
    }
}
