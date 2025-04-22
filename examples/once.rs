//! A timer that runs just a single time.
//!
//! This should result in an output of:
//! ```text
//! [t=0] Timer started
//! [t=1] Interval #1
//! [t=1] Timer finished
//! ```

use bevy::{log::LogPlugin, prelude::*};
use bevy_mod_observable_timer::*;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            ObservableTimerPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands
        // Shortcut for `ObservableTimer::from_seconds(1, 1.0)`
        .spawn(ObservableTimer::from_seconds(1.0, TimerMode::Once))
        .observe(|_: Trigger<TimerStarted>| {
            info!("Timer started");
        })
        .observe(|_: Trigger<TimerFinished>| {
            info!("Timer finished");
        })
        .observe(
            |_: Trigger<TimerStopped>, mut app_exit: EventWriter<AppExit>| {
                info!("Timer stopped");
                app_exit.write_default();
            },
        );
}
