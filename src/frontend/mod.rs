// use bevy_dioxus_hooks::{BevyValue, resource::hook::use_bevy_resource};
use bevy_dioxus_sync::panels::DioxusElementMarker;
// use crossbeam::channel::Receiver;
use dioxus::prelude::*;
// use std::{any::TypeId, ops::Deref, time::Duration};

#[derive(Debug)]
pub struct AppUi {
    // pub idle_time: Receiver<IdleTimeSample>,
    // pub automation_speed: Receiver<AutomationSpeedSample>,
}

impl DioxusElementMarker for AppUi {
    fn element(&self) -> Element {
        // game_ui(self.idle_time.clone(), self.automation_speed.clone())
        game_ui()
    }
}

pub fn game_ui(// idle_time: Receiver<IdleTimeSample>,
    // speed_samples: Receiver<AutomationSpeedSample>,
) -> Element {
    // let idle_time_res = use_bevy_resource::<CurrentIdleTimeSeconds>();
    // let best_idle_time_res = use_bevy_resource::<LongestIdleTimeSeconds>();
    // let window_size = use_bevy_resource::<WResolution>();
    // let mut max_idle_time: Signal<f32> = use_signal(|| 0.0);
    // let mut idle_times: Signal<Vec<_>> =
    //     use_signal(|| vec![IdleTimeSample::new(0.0), IdleTimeSample::new(0.0)]);
    // let mut automation_speed_samples: Signal<Vec<_>> = use_signal(|| {
    //     vec![
    //         AutomationSpeedSample::new(0.0),
    //         AutomationSpeedSample::new(0.0),
    //     ]
    // });
    // let mut max_automation_speed: Signal<f32> = use_signal(|| 0.0);
    //
    // let _idle_time_th = spawn(async move {
    //     loop {
    //         while let Ok(sample) = idle_time.try_recv() {
    //             idle_times.write().push(sample);
    //             idle_times
    //                 .write()
    //                 .retain(|time| time.when.elapsed() < IDLE_SAMPLE_WINDOW);
    //
    //             *max_idle_time.write() = idle_times
    //                 .read()
    //                 .iter()
    //                 .max_by(|a, b| a.time.total_cmp(&b.time).then(std::cmp::Ordering::Less))
    //                 .map(|time| time.time)
    //                 .unwrap_or(1.0) as f32;
    //         }
    //
    //         portable_async_sleep::async_sleep(Duration::from_secs_f32(0.25)).await;
    //     }
    // });
    //
    // let _speed_th = spawn(async move {
    //     loop {
    //         while let Ok(sample) = speed_samples.try_recv() {
    //             automation_speed_samples.write().push(sample);
    //             automation_speed_samples
    //                 .write()
    //                 .retain(|time| time.when.elapsed() < IDLE_SAMPLE_WINDOW);
    //
    //             *max_automation_speed.write() = idle_times
    //                 .read()
    //                 .iter()
    //                 .max_by(|a, b| a.time.total_cmp(&b.time).then(std::cmp::Ordering::Less))
    //                 .map(|time| time.time)
    //                 .unwrap_or(1.0) as f32;
    //         }
    //
    //         portable_async_sleep::async_sleep(Duration::from_secs_f32(0.25)).await;
    //     }
    // });

    rsx! {
        document::Stylesheet { href: asset!("src/frontend/ui.css") }

        main {
        }
    }
}
