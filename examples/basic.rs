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
        .spawn(ObservableTimer::from_seconds(5, 1.0))
        .observe(|_: Trigger<TimerStarted>| {
            info!("Timer started");
        })
        .observe(|trigger: Trigger<TimerInterval>| {
            info!("Interval #{}", trigger.event().count());
        })
        .observe(|_: Trigger<TimerFinished>, mut app_exit: EventWriter<AppExit>| {
            info!("Timer finished");
            app_exit.send_default();
        });
}
