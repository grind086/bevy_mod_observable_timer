# bevy_mod_observable_timer

This crate provides an observer-based timer system for bevy entities. Any entity may have an [`ObservableTimer`]
component attached to it, which will produce observable lifetime cycle triggers. Each timer is given an interval
duration, and may run for one or more intervals (including indefinitely).

- [`TimerStarted`] is triggered immediately after inserting a new `ObservableTimer` (including when overwriting
  an old one).
- [`TimerInterval`] is triggered after each elapsed interval.
- [`TimerFinished`] is triggered after the final interval elapses, or when the `ObservableTimer` component is
  removed/despawned.

When a timer finishes it will automatically perform some behavior. By default this is despawning its attached entity.
See [`TimerFinishBehavior`] for more information.

## Basic Example

```rust
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
```

Output:
```text
[t=0] Timer started
[t=1] Interval #1
[t=2] Interval #2
[t=3] Interval #3
[t=4] Interval #4
[t=5] Interval #5
[t=5] Timer finished
```