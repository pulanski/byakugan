// use std::time::Duration;

// use futures::stream::{
//     self,
//     StreamExt,
// };
// use indicatif::{
//     ProgressBar,
//     ProgressStyle,
// };
// use rand::thread_rng;
// use rand::Rng;
// use tokio::sync::mpsc;
// use tracing::info;
// use tracing::info_span;
// use tracing::instrument;

// #[instrument]
// async fn build_sub_unit(sub_unit: u64) {
//     let sleep_time =
//         thread_rng().gen_range(Duration::from_millis(5000)..
// Duration::from_millis(10000));     tokio::time::sleep(sleep_time).await;

//     if thread_rng().gen_bool(0.2) {
//         info!("sub_unit did something!");
//     }
// }

// #[instrument]
// async fn build(unit: u64, tx: mpsc::Sender<()>) {
//     let sleep_time =
//         thread_rng().gen_range(Duration::from_millis(2500)..
// Duration::from_millis(5000));     tokio::time::sleep(sleep_time).await;

//     let rand_num: f64 = thread_rng().gen();

//     if rand_num < 0.1 {
//         tokio::join!(build_sub_unit(0), build_sub_unit(1),
// build_sub_unit(2));     } else if rand_num < 0.3 {
//         tokio::join!(build_sub_unit(0), build_sub_unit(1));
//     } else {
//         build_sub_unit(0).await;
//     }

//     let _ = tx.send(()).await;
// }

// // Desired output:
// // Command: `build`.
// // Jobs: In progress: 17. Finished: 3847. Cache hits: 0%. Time elapsed: 38.8s

// #[tokio::main]
// async fn main() {
//     let (tx, mut rx) = mpsc::channel::<()>(20);
//     let progress_style = ProgressStyle::default_bar()
//         .template(
//             "Command: `build`. Jobs: In progress: {pos}. Finished: {len}.
// Cache hits: 0%. Time \              elapsed: {elapsed_precise}",
//         )
//         .expect("Failed to initialize progress bar UI.")
//         .progress_chars("##-");

//     let pb = ProgressBar::new(20);
//     pb.set_style(progress_style);

//     for i in 0..20 {
//         let tx_clone = tx.clone();
//         tokio::spawn(async move {
//             build(i, tx_clone).await;
//         });
//     }

//     let mut completed_jobs = 0;

//     while completed_jobs < 20 {
//         if let Some(()) = rx.recv().await {
//             completed_jobs += 1;
//             pb.inc(1);
//         }
//     }

//     pb.finish_with_message("Completed.");
// }
