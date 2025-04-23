//! A timer that runs indefinitely (this example does not exit).
//! 
//! This should result in an output of:
//! ```text
//! Timer started
//! Timer finished (#1)
//! Timer finished (#2)
//! Timer finished (#3)
//! Timer finished (#4)
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
            *count += 1;
            info!("Timer finished (#{})", *count);
        })
        // This will only run if the timer is manually cancelled
        .observe(|_: Trigger<TimerStopped>, mut app_exit: EventWriter<AppExit>| {
            info!("Timer stopped");
            app_exit.send_default();
        });
}
