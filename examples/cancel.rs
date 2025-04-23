//! Demonstrates cancelling a timer.
//!
//! This should result in an output of:
//! ```text
//! Timer started
//! Timer finished (#1)
//! Timer finished (#2)
//! Timer stopped (finished = false)
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
    let timer_id = commands
        .spawn(ObservableTimer::from_seconds(1.0, TimerMode::Repeating))
        .observe(|_: Trigger<TimerStarted>| {
            info!("Timer started");
        })
        .observe(|_: Trigger<TimerFinished>, mut count: Local<usize>| {
            *count += 1;
            info!("Timer finished (#{})", *count);
        })
        .observe(
            |trigger: Trigger<TimerStopped>, mut app_exit: EventWriter<AppExit>| {
                info!(
                    "Timer stopped (finished = {})",
                    trigger.event().finished
                );
                app_exit.send_default();
            },
        )
        .id();

    // We'll use another timer to cancel the first one after 2.5 seconds.
    commands
        .spawn(ObservableTimer::from_seconds(2.5, TimerMode::Once))
        .observe(move |_: Trigger<TimerFinished>, mut commands: Commands| {
            commands.entity(timer_id).despawn();
        });
}
