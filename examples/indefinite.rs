//! A timer that runs indefinitely (this example does not exit).
//! 
//! This should result in an output of:
//! ```text
//! [t=0] Timer started
//! [t=1] Timer finished (#1)
//! [t=1] Timer finished (#2)
//! [t=1] Timer finished (#3)
//! [t=1] Timer finished (#4)
//! ...
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
        // Shortcut for `ObservableTimer::from_seconds(0, 1.0)`
        .spawn(ObservableTimer::from_seconds(1.0, TimerMode::Repeating))
        .observe(|_: Trigger<TimerStarted>| {
            info!("Timer started");
        })
        .observe(|_: Trigger<TimerFinished>, mut count: Local<usize>| {
            info!("Timer finished (#{})", *count);
            *count += 1;
        })
        // This will only run if the timer is manually cancelled
        .observe(|_: Trigger<TimerFinished>, mut app_exit: EventWriter<AppExit>| {
            info!("Timer stopped");
            app_exit.write_default();
        });
}
