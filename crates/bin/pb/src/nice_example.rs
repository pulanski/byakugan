use std::ops::Add;
use std::time::Duration;

use derive_more::Display;
use futures::stream::{
    self,
    StreamExt,
};
use indicatif::ProgressState;
use indicatif::ProgressStyle;
use owo_colors::OwoColorize;
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

fn elapsed_subsec(state: &ProgressState, writer: &mut dyn std::fmt::Write) {
    let seconds = state.elapsed().as_secs();
    let sub_seconds = (state.elapsed().as_millis() % 1000) / 100;
    let _ = writer.write_str(&format!("{seconds}.{sub_seconds}s"));
}

#[instrument]
async fn build_sub_unit(sub_unit: u64) {
    let sleep_time =
        thread_rng().gen_range(Duration::from_millis(5000)..Duration::from_millis(10000));
    tokio::time::sleep(sleep_time).await;

    if thread_rng().gen_bool(0.2) {
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

pub fn long_running_task_msg() -> String {
    let mut msg = "Long running task".red().to_string();
    msg = msg.add("... (".black().to_string().as_str());
    msg = msg
        .add("Press Ctrl+C to cancel".bright_yellow().italic().to_string().as_str())
        .add(")".black().to_string().as_str());
    msg.into()
}

pub fn medium_running_task_msg() -> String {
    let mut msg = "Medium running task".yellow().to_string();
    msg = msg.add("... (".black().to_string().as_str());
    msg = msg
        .add("Press Ctrl+C to cancel".bright_yellow().italic().to_string().as_str())
        .add(")".black().to_string().as_str());
    msg.into()
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
            .unwrap()
            .with_key("elapsed_subsec", elapsed_subsec)
            .with_key("color_start", |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                let elapsed = state.elapsed();

                if elapsed > Duration::from_secs(8) {
                    // Red
                    let _ = write!(writer, "{} ", long_running_task_msg());
                    // let _ = write!(writer, "\x1b[{}m", 1 + 30);
                } else if elapsed > Duration::from_secs(4) {
                    // Yellow
                    let _ = write!(writer, "{} ", medium_running_task_msg());
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
        .with_span_child_prefix_symbol("↳ ")
        .with_span_child_prefix_indent(" ");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .init();

    let ulid = ulid::Ulid::new();
    tracing::info!("BUILD ID: {}", ulid.to_string().green().italic());

    // Desired output:
    // Command: `build`.
    // Jobs: In progress: 17. Finished: 3847. Cache hits: 0%. Time elapsed: 38.8s

    let template = "Working on tasks for command: `build`. {wide_msg}
{elapsed_subsec}\n{wide_bar}";

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
