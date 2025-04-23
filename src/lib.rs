#![doc = include_str!("../README.md")]

use core::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use bevy::{
    ecs::{
        component::ComponentId,
        schedule::{InternedScheduleLabel, ScheduleLabel},
        world::DeferredWorld,
    },
    prelude::*,
};

/// The [`SystemSet`] during which [`ObservableTimer`]s are updated.
///
/// Runs in [`Update`] by default, but this is configurable. See [`ObservableTimerPlugin::in_schedule()`].
#[derive(SystemSet, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservableTimerSystems;

/// This plugin provides functionality for the [`ObservableTimer`] component.
///
/// See the crate-level documentation for more information.
pub struct ObservableTimerPlugin {
    schedule: InternedScheduleLabel,
}

impl ObservableTimerPlugin {
    /// Creates an `ObservableTimerPlugin` whose timers update in the given schedule.
    ///
    /// The default plugin updates in [`Update`].
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_mod_observable_timer::*;
    /// # let mut app = App::new();
    /// // Timers will be updated in `Last`
    /// app.add_plugins(ObservableTimerPlugin::in_schedule(Last));
    /// ```
    pub fn in_schedule(schedule: impl ScheduleLabel) -> Self {
        Self {
            schedule: schedule.intern(),
        }
    }
}

impl Default for ObservableTimerPlugin {
    fn default() -> Self {
        Self::in_schedule(Update)
    }
}

impl Plugin for ObservableTimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            self.schedule,
            update_observable_timers.in_set(ObservableTimerSystems),
        );
    }
}

/// Describes the behavior that should be taken by an [`ObservableTimer`] upon finishing.
///
/// # See also
/// - [`ObservableTimer::with_finish_behavior()`]
/// - [`ObservableTimer::finish_behavior`]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TimerFinishBehavior {
    /// Do nothing.
    ///
    /// Note that this will leave the `ObservableTimer` component in place, which means it will still be looped through
    /// when updating timers.
    None,
    /// Remove only the `ObservableTimer` component.
    RemoveComponent,
    /// Despawn the entity that the `ObservableTimer` is attached to.
    ///
    /// This is the default behavior.
    #[default]
    DespawnEntity,
}

/// A timer component that triggers observable lifecycle events on its [`Entity`].
///
/// When an `ObservableTimer` is first added to an `Entity` (either by adding a new one, or replacing the current one)
/// a [`TimerStarted`] event will be triggered. Then, each time an interval completes, a [`TimerFinished`] event will
/// be triggered. Finally, when the timer component is removed, a [`TimerStopped`] event will be triggered.
///
/// By default a [`TimerMode::Once`] timer will despawn its `Entity` when it finishes. This behavior can be changed to
/// removing only the `ObservableTimer` component, or doing nothing. See [`Self::with_finish_behavior`] for setting
/// behavior at creation, or [`Self::finish_behavior`] for changing it after creation. Note that this behavior will
/// not be run if the timer is removed manually before finishing.
///
/// To cancel a currently running timer simply remove the component. This will cause a [`TimerStopped`] event to be
/// triggered.
#[derive(Component, Debug, Clone)]
#[component(on_remove = on_timer_removed)]
pub struct ObservableTimer {
    /// The internal [`Timer`].
    pub timer: Timer,
    /// The timer's [finish behavior](TimerFinishBehavior).
    pub finish_behavior: TimerFinishBehavior,
}

impl ObservableTimer {
    /// Create a new timer.
    pub fn new(duration: Duration, mode: TimerMode) -> Self {
        Self {
            timer: Timer::new(duration, mode),
            finish_behavior: TimerFinishBehavior::default(),
        }
    }

    /// Create a new timer from a duration in seconds.
    pub fn from_seconds(duration: f32, mode: TimerMode) -> Self {
        Self {
            timer: Timer::from_seconds(duration, mode),
            finish_behavior: TimerFinishBehavior::default(),
        }
    }

    /// Set the [`TimerFinishBehavior`] for this timer.
    pub fn with_finish_behavior(self, finish_behavior: TimerFinishBehavior) -> Self {
        Self {
            finish_behavior,
            ..self
        }
    }
}

impl Deref for ObservableTimer {
    type Target = Timer;
    fn deref(&self) -> &Self::Target {
        &self.timer
    }
}

impl DerefMut for ObservableTimer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.timer
    }
}

/// A timer [`Event`] that is triggered when an [`ObservableTimer`] is inserted or spawned.
#[derive(Event, Debug)]
pub struct TimerStarted;

/// A timer [`Event`] that is triggered when an [`ObservableTimer`] is removed or despawned.
#[derive(Event, Debug)]
pub struct TimerStopped {
    /// This is `true` for [`TimerMode::Once`] timers that finished normally, and removed or
    /// despawned themselves.
    pub finished: bool,
}

/// A timer [`Event`] that is triggered when an [`ObservableTimer`] finishes.
#[derive(Event, Debug)]
pub struct TimerFinished;

fn on_timer_removed(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    let timer = world.get::<ObservableTimer>(entity).unwrap();
    let finished = timer.mode() == TimerMode::Once && timer.finished();
    world
        .commands()
        .trigger_targets(TimerStopped { finished }, entity);
}

fn update_observable_timers(
    time: Res<Time>,
    mut timers: Query<(Entity, &mut ObservableTimer)>,
    mut commands: Commands,
) {
    let delta = time.delta();
    for (entity, mut timer) in timers.iter_mut() {
        if timer.is_added() {
            commands.trigger_targets(TimerStarted, entity)
        }

        if timer.tick(delta).just_finished() {
            for _ in 0..timer.times_finished_this_tick() {
                commands.trigger_targets(TimerFinished, entity);
            }

            if timer.mode() == TimerMode::Once {
                match timer.finish_behavior {
                    TimerFinishBehavior::None => {}
                    TimerFinishBehavior::RemoveComponent => {
                        commands.entity(entity).remove::<ObservableTimer>();
                    }
                    TimerFinishBehavior::DespawnEntity => {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}
