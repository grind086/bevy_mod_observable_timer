# bevy_mod_observable_timer

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/grind086/bevy_mod_observable_timer#license)
[![Crates.io](https://img.shields.io/crates/v/bevy_mod_observable_timer.svg)](https://crates.io/crates/bevy_mod_observable_timer)
[![Downloads](https://img.shields.io/crates/d/bevy_mod_observable_timer.svg)](https://crates.io/crates/bevy_mod_observable_timer)
[![Docs](https://docs.rs/bevy_mod_observable_timer/badge.svg)](https://docs.rs/bevy_mod_observable_timer/latest/bevy_mod_observable_timer/)

This crate provides an observer-based timer system for bevy entities. Any entity may have an [`ObservableTimer`]
component attached to it, which will produce observable lifetime cycle triggers.

- [`TimerStarted`] is triggered immediately after inserting a new `ObservableTimer` (including when overwriting
  an old one).
- [`TimerFinished`] is triggered after each elapsed interval.
- [`TimerStopped`] is triggered when the `ObservableTimer` component is removed/despawned.

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
        .spawn(ObservableTimer::from_seconds(1.0, TimerMode::Repeating))
        .observe(|_: Trigger<TimerStarted>| {
            info!("Timer started");
        })
        .observe(
            |trigger: Trigger<TimerFinished>, mut count: Local<usize>, mut commands: Commands| {
                *count += 1;
                info!("Timer finished (#{})", *count);

                if *count == 5 {
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
```

Output:
```text
Timer started
Timer finished (#1)
Timer finished (#2)
Timer finished (#3)
Timer finished (#4)
Timer finished (#5)
Timer stopped
```

## Compatibility

| bevy   | bevy_mod_observable_timer |
| ------ | ------------------------- |
| `0.16` | `0.3`                     |
| `0.15` | `0.2`                     |
| `0.14` | `0.1`                     |

## License

Except where noted, all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
