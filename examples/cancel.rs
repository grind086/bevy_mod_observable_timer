//! Demonstrates cancelling a timer.

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
            info!("Timer finished (#{})", *count);
            *count += 1;
        })
        .observe(
            |trigger: Trigger<TimerStopped>, mut app_exit: EventWriter<AppExit>| {
                info!(
                    "Timer stopped (finished = {})",
                    trigger.event().finished
                );
                app_exit.write_default();
            },
        )
        .id();

    commands
        .spawn(ObservableTimer::from_seconds(2.5, TimerMode::Once))
        .observe(move |_: Trigger<TimerFinished>, mut commands: Commands| {
            commands.entity(timer_id).despawn();
        });
}
