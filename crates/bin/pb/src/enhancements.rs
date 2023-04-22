// fn color_from(s: &str) -> Rgb {
//     // Map each character in the string to an HSV value
//     // let hues = s.chars().map(|c| c as f32 / 255.0);
//     // let hsv = Hsv::new(hues.fold(0.0, |a, b| a + b), 1.0, 1.0);
//     // // Convert the HSV value to an RGB value
//     // Rgb::from_hsv(hsv)
//     // TODO
//     Rgb(0, 0, 0)
// }
// NOTE: AFTER

// TODO
// pub fn medium_running_task_msg(task_id: TaskId (Ulid `clamp len to 5`)) ->
// String {

// pub fn short_running_task_msg(duration: Duration, task_id: String) -> String
// {     let elapsed_secs = duration.as_secs_f64();
//     let dark_green = Rgb(0, 100, 0);
//     let green = Rgb(0, 255, 0);
//     let gradient = interpolate_color(&dark_green, &green, elapsed_secs /
// 3.0);     format!(
//         "{} {}{}{} {}",
//         "Short task".color(gradient).italic(),
//         "[".black().italic(),
//         task_id.black().italic(),
//         "]".black().italic(),
//         "...".black(),
//     )
//     .into()
//     // format!("{}{}", "Short task".color(gradient).italic(),
//     // "...".black(),).into()
// }
