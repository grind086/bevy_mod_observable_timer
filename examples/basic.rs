//! The example from the README.

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
        .spawn(ObservableTimer::from_seconds(1.0, TimerMode::Repeating))
        .observe(|_: Trigger<TimerStarted>| {
            info!("Timer started");
        })
        .observe(
            |trigger: Trigger<TimerFinished>, mut count: Local<usize>, mut commands: Commands| {
                info!("Timer finished (#{})", *count);
                *count += 1;

                if *count >= 5 {
                    commands.entity(trigger.target()).despawn();
                }
            },
        )
        .observe(
            |_: Trigger<TimerStopped>, mut app_exit: EventWriter<AppExit>| {
                info!("Timer stopped");
                app_exit.write_default();
            },
        );
}
