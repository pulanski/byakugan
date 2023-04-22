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
use parking_lot::RwLock;
use rand::thread_rng;
use rand::Rng;
use shrinkwraprs::Shrinkwrap;
use smartstring::alias::String;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tracing::info;
use tracing::info_span;
use tracing::instrument;
use tracing::Subscriber;
use tracing_indicatif::{
    span_ext::IndicatifSpanExt,
    IndicatifLayer,
};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{
    layer::SubscriberExt,
    registry::LookupSpan,
};
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
static TASK_COUNTER: Lazy<TaskTracker> = Lazy::new(|| TaskTracker::new(0, 0, 0));

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
    cache_hits:  usize,
}

#[derive(Debug, Clone, Shrinkwrap)]
struct TaskTracker(Arc<RwLock<TaskTrackerData>>);

impl TaskTracker {
    pub fn new(in_progress: usize, completed: usize, cache_hits: usize) -> Self {
        Self(Arc::new(RwLock::new(TaskTrackerData::new(in_progress, completed, cache_hits))))
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

fn in_progress(state: &ProgressState, writer: &mut dyn std::fmt::Write) {
    let task_counter = TASK_COUNTER.read();

    let _ = write!(writer, "{}", task_counter.in_progress);
}

fn completed(state: &ProgressState, writer: &mut dyn std::fmt::Write) {
    let task_counter = TASK_COUNTER.read();

    let _ = write!(writer, "{}", task_counter.completed);
}

fn cache_hits_pct(state: &ProgressState, writer: &mut dyn std::fmt::Write) {
    let percentage = cache_hits_percentage();
    let percentage_msg = format!("{}", cache_hits_msg(percentage));
    let _ = write!(writer, "{percentage_msg}");
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

pub fn increment_cache_hits() {
    let mut task_counter = TASK_COUNTER.write();
    *task_counter.cache_hits_mut() += 1;
    tracing::trace!(
        "Cache hit, in progress: {}, completed: {}, cache hits: {}",
        task_counter.in_progress(),
        task_counter.completed(),
        task_counter.cache_hits()
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
#[cached(size = 100)]
async fn build(unit: usize) {
    increment_in_progress_task(); // Increment in-progress tasks when a new task starts

    let mut tasks = Vec::new();

    let sleep_time =
        thread_rng().gen_range(Duration::from_millis(2500)..Duration::from_millis(5000));
    tokio::time::sleep(sleep_time).await;

    let rand_num: f64 = thread_rng().gen();

    if rand_num < 0.1 {
        tasks.push(tokio::spawn(build_sub_unit(0)));
        tasks.push(tokio::spawn(build_sub_unit(1)));
        tasks.push(tokio::spawn(build_sub_unit(2)));
    } else if rand_num < 0.3 {
        tasks.push(tokio::spawn(build_sub_unit(0)));
        tasks.push(tokio::spawn(build_sub_unit(1)));
    } else if rand_num < 0.6 {
        tasks.push(tokio::spawn(build_sub_unit(0)));
    } else {
        tasks.push(tokio::spawn(build_sub_unit(0)));
        tasks.push(tokio::spawn(build_sub_unit(1)));
        tasks.push(tokio::spawn(build_sub_unit(2)));
        tasks.push(tokio::spawn(build_sub_unit(3)));
        tasks.push(tokio::spawn(build_sub_unit(4)));
    }

    for task in tasks {
        // Get the current cache size
        let cache_size_before = BUILD_SUB_UNIT.lock().await.cache_size();

        task.await.expect("Task failed");

        // Get the new cache size
        let cache_size_after = BUILD_SUB_UNIT.lock().await.cache_size();

        // If the cache size has not changed, it means there was a cache hit
        if cache_size_before == cache_size_after {
            increment_cache_hits();
        }

        increment_completed_task(); // Increment completed tasks when a
                                    // task is successfully completed
    }
}

fn display_diagnostics(task_display: String, start_time: Instant) {
    let task_counter = TASK_COUNTER.read();

    info!("Finished executing tasks for command: {}", task_display);
    info!(" Jobs Finished{} {}", ":".black(), task_counter.completed.green().bold().italic());
    info!(" {}{}", cache_hits_msg(cache_hits_percentage()), "%".black());
    info!(
        " Time elapsed{} {}{}{}s",
        ":".black(),
        start_time.elapsed().as_secs().to_string().cyan().italic(),
        ".".black(),
        start_time.elapsed().subsec_millis().to_string().cyan().italic()
    );
}

fn create_task_display(task: &str, task_id: &str) -> String {
    format!(
        "{}{}{} {}{}{}",
        "`".red(),
        task.green(),
        "`".red(),
        "[".black(),
        task_id[..5].to_string().cyan().italic(),
        "]".black()
    )
    .into()
}

fn prettify_template(template: &str, task_display: &str) -> String {
    template
        .replace("{task_display}", task_display)
        .replace('.', &format!("{}", ".".black()))
        .replace('%', &format!("{}", "%".black()))
        .replace(':', &format!("{}", ":".black()))
        .replace("In progress", &format!("{}", "In progress".bright_yellow()))
        .replace("Finished", &format!("{}", "Finished".green()))
        .into()
}

async fn execute_build_tasks(num_units: usize) {
    stream::iter((0..num_units).map(build)).buffer_unordered(7).collect::<Vec<()>>().await;
}

fn create_header_span(task_display: &str) -> tracing::Span {
    let template = "Executing tasks for command: {task_display}. {wide_msg} Jobs: In progress: \
                    {in_progress}. Finished: {completed}. {cache_hits_percentage}%. Time elapsed: \
                    {elapsed_subsec}
\n{wide_bar}";

    let template = prettify_template(template, task_display);
    let header_span = info_span!("header");
    header_span.pb_set_style(
        &ProgressStyle::with_template(&template)
            .unwrap()
            .with_key("elapsed_subsec", elapsed_subsec)
            .with_key("in_progress", in_progress)
            .with_key("completed", completed)
            .with_key("cache_hits_percentage", cache_hits_pct)
            .progress_chars("---"),
    );
    header_span
}

fn create_task_id() -> String {
    let ulid = Ulid::new();
    ulid.to_string()[..10].to_string().into()
}

fn color_end(state: &ProgressState, writer: &mut dyn std::fmt::Write) {
    if state.elapsed() > Duration::from_secs(4) {
        let _ = write!(writer, "\x1b[0m");
    }
}

fn create_progress_style() -> ProgressStyle {
    ProgressStyle::with_template(
        r"{spinner:.green}{color_start}{span_child_prefix}{span_fields} -- {span_name}{wide_msg}{elapsed_subsec}{color_end}",
    )
    .expect("Failed to initialize TUI")
    .tick_strings(&["◐", "◓", "◑", "◒"])
    .with_key("elapsed_subsec", elapsed_subsec)
    .with_key("color_start", task_msg_display)
    .with_key("color_end", color_end)
}

fn create_indicatif_layer<S>() -> IndicatifLayer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    IndicatifLayer::new()
        .with_progress_style(create_progress_style())
        .with_span_child_prefix_symbol(&RIGHT_ARROW_SYMBOL)
        .with_span_child_prefix_indent(" ")
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

fn cache_hits_percentage() -> f64 {
    let task_counter = TASK_COUNTER.read();
    if task_counter.completed() > &0 {
        (*task_counter.cache_hits() as f64 / *task_counter.completed() as f64) * 100.0
    } else {
        0.0
    }
}

#[tokio::main]
async fn main() {
    let start_time = Instant::now();
    let num_units = 10;
    let task = "build";

    let indicatif_layer = create_indicatif_layer();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(indicatif_layer.get_stderr_writer())
                .fmt_fields(tracing_subscriber::fmt::format::DefaultFields::new())
                .without_time()
                .with_target(true)
                .with_ansi(true)
                .with_timer(tracing_subscriber::fmt::time::Uptime::default()),
        )
        .with(indicatif_layer)
        .init();

    let task_id = create_task_id();
    info!("{} ID: {}", task, task_id.cyan().italic());

    let task_display = create_task_display(task, &task_id);
    let header_span = create_header_span(&task_display);
    header_span.pb_start();

    // Display full "-----" line underneath the header.
    header_span.pb_set_length(1);
    header_span.pb_set_position(1);

    execute_build_tasks(num_units).await;

    display_diagnostics(task_display, start_time);
}

// phases:
// - scheduling
//   - lays out an acyclic task graph
// - executing
//   - runs the task graph
//     - runs tasks concurrently using a thread pool of configurable size
//       (default: number of logical cores)
//     - tasks
