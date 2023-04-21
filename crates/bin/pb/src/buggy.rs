// use std::ops::Add;
// use std::time::Duration;

// use derive_more::Display;
// use futures::stream::{
//     self,
//     StreamExt,
// };
// use indicatif::{
//     MultiProgress,
//     ProgressBar,
//     ProgressStyle,
// };
// use owo_colors::OwoColorize;
// use rand::thread_rng;
// use rand::Rng;
// use smartstring::alias::String;
// use tracing::{
//     info,
//     instrument,
//     Instrument,
// };
// use ulid::Ulid;

// async fn build_sub_unit(sub_unit: u64) {
//     let sleep_time =
//         thread_rng().gen_range(Duration::from_millis(5000)..
// Duration::from_millis(10000));     tokio::time::sleep(sleep_time).await;

//     if thread_rng().gen_bool(0.2) {
//         info!("sub_unit did something!");
//     }
// }

// #[derive(Debug, Clone, Copy, Display)]
// #[display(fmt = "rudolph")]
// pub enum TaskKind {
//     Build,
//     Run,
//     Test,
//     Embed,
//     Cache,
//     Clean,
//     DBRead,
// }

// #[instrument]
// async fn build(unit: u64, progress_bars: &MultiProgress) {
//     let sleep_time =
//         thread_rng().gen_range(Duration::from_millis(2500)..
// Duration::from_millis(5000));     let task_name =
// format!("root//third-party:task-{}", unit);

//     // let mut pb = progress_bars.add(ProgressBar::new_spinner());
//     // pb = pb
//     //     .set_style(
//     //         ProgressStyle::default_spinner()
//     //             .template("{prefix} {wide_msg:.cyan} {elapsed_precise}")
//     //             .expect("Failed to initialize TUI."),
//     //     )
//     //     .set_prefix(&task_name);
//     // pb.set_prefix(&task_name);

//     pb.tick();
//     pb.set_message(
//         "action (rustc rlib-static-static-metadata/portable_atomic-metadata
// rlib,static,metadata \          [diag]...",
//     );
//     tokio::time::sleep(sleep_time).await;

//     pb.finish_with_message("finished");

//     let rand_num: f64 = thread_rng().gen();

//     if rand_num < 0.1 {
//         tokio::join!(build_sub_unit(0), build_sub_unit(1),
// build_sub_unit(2));     } else if rand_num < 0.3 {
//         tokio::join!(build_sub_unit(0), build_sub_unit(1));
//     } else {
//         build_sub_unit(0).await;
//     }
// }

// #[tokio::main]
// async fn main() {
//     let ulid = Ulid::new();
//     info!("Build ID: {}", ulid.to_string().green().italic());

//     let template = "Command: `build`.
// Jobs: \                     In progress: 167. Finished: 3024. Cache hits: 0%.
// Time elapsed: 8.0s";     let progress_bars = MultiProgress::new();
//     let header_pb = ProgressBar::new_spinner();
//     // header_pb.set_style(
//     //     ProgressStyle::default_spinner()
//     //         .template("{spinner:.green} {msg:.bright_cyan}")
//     //         .expect("Failed to initialize TUI.")
//     //         .with_key(
//     //             "spinner",
//     //             ProgressStyle::default_spinner()
//     //                 .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦",
// "⠧", "⠇",     // "⠏"])                 .template("{spinner:.green}
// {msg:.bright_cyan}")     //                 .expect("Failed to initialize
// TUI."),     //         ),
//     //     // .with_spinner("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
//     // );
//     header_pb.set_message(template);
//     // progress_bars.push(header_pb);

//     let task_futures: Vec<_> = (0..20)
//         .map(|unit| {
//             let progress_bars = progress_bars.clone();
//             build(unit, &progress_bars)
//         })
//         .collect();

//     // progress_bars.join_and_clear().await;

//     for task_future in task_futures {
//         task_future.await;
//     }
// }
