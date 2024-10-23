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
        .spawn(ObservableTimer::from_seconds(5, 1.0))
        .observe(|_: Trigger<TimerStarted>| {
            info!("Timer started");
        })
        .observe(|trigger: Trigger<TimerInterval>| {
            info!("Interval #{}", trigger.event().count());
        })
        .observe(
            |trigger: Trigger<TimerFinished>, mut app_exit: EventWriter<AppExit>| {
                info!(
                    "Timer finished (cancelled = {})",
                    trigger.event().cancelled()
                );
                app_exit.send_default();
            },
        )
        .id();

    commands
        .spawn(ObservableTimer::once_from_seconds(2.5))
        .observe(move |_: Trigger<TimerFinished>, mut commands: Commands| {
            commands.entity(timer_id).despawn();
        });
}
