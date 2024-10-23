#![doc = include_str!("../README.md")]

use std::time::Duration;

use bevy::{
    ecs::{
        component::{ComponentHooks, ComponentId, StorageType},
        schedule::{InternedScheduleLabel, ScheduleLabel},
        world::DeferredWorld,
    },
    prelude::*,
};

/// The [`SystemSet`] during which [`ObservableTimer`]s are updated.
///
/// Runs in [`Update`] by default, but this is configurable. See [`ObservableTimerPlugin::in_schedule()`].
#[derive(SystemSet, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservableTimerSet;

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
            update_observable_timers.in_set(ObservableTimerSet),
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
    /// when updating timers. This may have performance implications if using a large number of timers with this
    /// behavior.
    None,
    /// Remove only the `ObservableTimer` component.
    RemoveComponent,
    /// Despawn the entity that the `ObservableTimer` is attached to.
    ///
    /// This is the default behavior.
    #[default]
    DespawnEntity,
    /// Despawn the entity that the `ObservableTimer` is attached to, along with its children.
    DespawnRecursive,
}

/// A timer component that triggers observable lifecycle events on its [`Entity`].
///
/// When an `ObservableTimer` is first added to an `Entity` (either by adding a new one, or replacing the current one)
/// a [`TimerStarted`] event will be triggered. Then, each time an interval completes, a [`TimerInterval`] event will
/// be triggered. Finally, when the timer is finished or its component is removed, a [`TimerFinished`] event will be
/// triggered.
///
/// By default the timer will despawn its `Entity` when it finishes. This behavior can be changed to removing only the
/// `ObservableTimer` component, or recursively despawning children. See [`Self::with_finish_behavior`] for setting
/// behavior at creation, or [`Self::finish_behavior`] for changing it after creation. Note that this behavior will
/// not be run if the timer is removed manually before finishing.
///
/// To cancel a currently running timer simply remove the component. This will cause a [`TimerFinished`] event to be
/// triggered with [`TimerFinished::cancelled()`] set to `true`.
#[derive(Debug, Clone)]
pub struct ObservableTimer {
    timer: Timer,
    remaining_intervals: Option<u32>,
    elapsed_intervals: u32,
    intervals_this_tick: u32,
    /// The timer's [finish behavior](TimerFinishBehavior).
    pub finish_behavior: TimerFinishBehavior,
}

impl ObservableTimer {
    /// Create a new timer that will run for `interval_count` intervals of length `interval_duration`.
    ///
    /// An `interval_count` of `0` will result in a timer that runs indefinitely.
    pub fn new(interval_count: u32, interval_duration: Duration) -> Self {
        let (timer_mode, remaining_intervals) = match interval_count {
            0 => (TimerMode::Repeating, None),
            1 => (TimerMode::Once, Some(1)),
            n => (TimerMode::Repeating, Some(n)),
        };
        Self {
            timer: Timer::new(interval_duration, timer_mode),
            remaining_intervals,
            elapsed_intervals: 0,
            intervals_this_tick: 0,
            finish_behavior: TimerFinishBehavior::default(),
        }
    }

    /// Create a new timer that will run for `interval_count` intervals of length `interval_seconds`.
    ///
    /// An `interval_count` of `0` will result in a timer that runs indefinitely.
    pub fn from_seconds(interval_count: u32, interval_seconds: f32) -> Self {
        Self::new(interval_count, Duration::from_secs_f32(interval_seconds))
    }

    /// Create a new timer that will run for at most `total_duration` in intervals of length `interval_duration`.
    pub fn from_total(total_duration: Duration, interval_duration: Duration) -> Self {
        let interval_count =
            (total_duration.as_secs_f32() / interval_duration.as_secs_f32()).floor() as _;
        Self::new(interval_count, interval_duration)
    }

    /// Create a new timer that will run for at most `total_seconds` in intervals of length `interval_seconds`.
    ///
    /// ```ignore
    /// // Emits a `TimerInterval` every 5 seconds for 60 seconds
    /// ObservableTimer::from_total_seconds(60.0, 5.0);
    /// ```
    pub fn from_total_seconds(total_seconds: f32, interval_seconds: f32) -> Self {
        Self::from_total(
            Duration::from_secs_f32(total_seconds),
            Duration::from_secs_f32(interval_seconds),
        )
    }

    /// Creates a new timer with a single interval.
    ///
    /// ```ignore
    /// ObservableTimer::once(Duration::from_secs_f32(5.0));
    /// ```
    pub fn once(duration: Duration) -> Self {
        Self::new(1, duration)
    }

    /// Creates a new timer with a single interval.
    ///
    /// ```ignore
    /// ObservableTimer::once_from_seconds(5.0);
    /// ```
    pub fn once_from_seconds(seconds: f32) -> Self {
        Self::from_seconds(1, seconds)
    }

    /// Creates a new timer that runs indefinitely with intervals of the given length.
    pub fn indefinite(interval_duration: Duration) -> Self {
        Self::new(1, interval_duration)
    }

    /// Creates a new timer that runs indefinitely with intervals of the given length.
    pub fn indefinite_from_seconds(interval_seconds: f32) -> Self {
        Self::from_seconds(0, interval_seconds)
    }

    /// Sets the timer's [finish behavior](TimerFinishBehavior).
    pub fn with_finish_behavior(mut self, finish_behavior: TimerFinishBehavior) -> Self {
        self.finish_behavior = finish_behavior;
        self
    }
}

impl ObservableTimer {
    /// Returns `true` if the timer is paused.
    pub fn paused(&self) -> bool {
        self.timer.paused()
    }

    /// Pauses the timer.
    pub fn pause(&mut self) {
        self.timer.pause();
    }

    /// Resumes a paused timer.
    pub fn unpause(&mut self) {
        self.timer.unpause();
    }

    /// Whether the timer is finished running.
    pub fn is_done(&self) -> bool {
        self.remaining_intervals == Some(0)
    }

    /// Returns `true` if the timer finished in the last tick.
    pub fn just_finished(&self) -> bool {
        self.is_done() && self.intervals_this_tick != 0
    }

    /// The number of intervals completed in the last tick.
    pub fn intervals_this_tick(&self) -> u32 {
        self.intervals_this_tick
    }

    /// The length of a single interval.
    pub fn interval_duration(&self) -> Duration {
        self.timer.duration()
    }

    /// The amount of time elapsed in the current interval.
    pub fn interval_elapsed(&self) -> Duration {
        if self.is_done() {
            Duration::ZERO
        } else {
            self.timer.elapsed()
        }
    }

    /// The fraction of the current interval's elapsed time (goes from `0.0` to `1.0`).
    pub fn interval_fraction(&self) -> f32 {
        if self.is_done() {
            0.0
        } else {
            self.timer.fraction()
        }
    }

    /// The amount of time remaining in the current interval.
    pub fn interval_remaining(&self) -> Duration {
        if self.is_done() {
            Duration::ZERO
        } else {
            self.timer.remaining()
        }
    }

    /// The fraction of the current interval's remaining time (goes from `1.0` to `0.0`).
    pub fn interval_fraction_remaining(&self) -> f32 {
        if self.is_done() {
            0.0
        } else {
            self.timer.fraction_remaining()
        }
    }

    /// The number of full intervals elapsed.
    pub fn elapsed_intervals(&self) -> u32 {
        self.elapsed_intervals
    }

    /// The number of elapsed intervals as a float. This includes the partially elapsed currently running interval.
    pub fn elapsed_intervals_f32(&self) -> f32 {
        if self.remaining_intervals == Some(0) {
            self.elapsed_intervals as _
        } else {
            self.elapsed_intervals as f32 + self.timer.fraction()
        }
    }

    /// The number of remaining intervals, rounded up to the nearest integer.
    pub fn remaining_intervals(&self) -> Option<u32> {
        self.remaining_intervals
    }

    /// The number of remaining intervals as a float.
    pub fn remaining_intervals_f32(&self) -> Option<f32> {
        self.remaining_intervals
            .map(|n| (n - 1) as f32 + self.timer.fraction_remaining())
    }

    /// The total duration of the timer across all intervals.
    ///
    /// This will be `None` for timers that repeat indefinitely.
    pub fn duration(&self) -> Option<Duration> {
        self.remaining_intervals
            .map(|n| self.timer.duration() * (n + self.elapsed_intervals))
    }

    /// The total elapsed duration on the timer.
    pub fn elapsed(&self) -> Duration {
        let full_elapsed = self.timer.duration() * self.elapsed_intervals;
        if self.remaining_intervals == Some(0) {
            full_elapsed
        } else {
            full_elapsed + self.timer.elapsed()
        }
    }

    /// The fraction of the timer's total elapsed time (goes from `0.0` to `1.0`).
    pub fn fraction(&self) -> f32 {
        match self.remaining_intervals {
            None => 0.0,
            Some(0) => 1.0,
            Some(n) => {
                (self.elapsed_intervals as f32 + self.timer.fraction())
                    / (self.elapsed_intervals + n) as f32
            }
        }
    }

    /// The total remaining duration on the timer.
    pub fn remaining(&self) -> Option<Duration> {
        match self.remaining_intervals {
            None => None,
            Some(0) => Some(Duration::ZERO),
            Some(n) => Some(self.timer.duration() * (n - 1) + self.timer.remaining()),
        }
    }

    /// The fraction of the timer's total remaining time (goes from `1.0` to `0.0`).
    pub fn fraction_remaining(&self) -> f32 {
        match self.remaining_intervals {
            None => 1.0,
            Some(0) => 0.0,
            Some(n) => {
                ((n - 1) as f32 + self.timer.fraction_remaining())
                    / (self.elapsed_intervals + n) as f32
            }
        }
    }

    /// Advance the timer by `delta`.
    fn tick(&mut self, delta: Duration) {
        match self.remaining_intervals {
            Some(0) => {
                self.intervals_this_tick = 0;
            }
            Some(remaining_intervals) => {
                let intervals_this_tick = self
                    .timer
                    .tick(delta)
                    .times_finished_this_tick()
                    .min(remaining_intervals);

                self.intervals_this_tick = intervals_this_tick;
                self.remaining_intervals = Some(remaining_intervals - intervals_this_tick);
                self.elapsed_intervals += intervals_this_tick;
            }
            None => {
                let intervals_this_tick = self.timer.tick(delta).times_finished_this_tick();

                self.intervals_this_tick = intervals_this_tick;
                self.elapsed_intervals += intervals_this_tick;
            }
        }
    }
}

impl Component for ObservableTimer {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(on_timer_removed);
    }
}

/// A timer [`Event`] that is triggered when an [`ObservableTimer`] is added or inserted.
#[derive(Event, Debug)]
pub struct TimerStarted {
    // This prevents the ZST from being instantiated outside this crate.
    _inner: (),
}

/// A timer [`Event`] that is triggered when an [`ObservableTimer`] interval has passed.
#[derive(Event, Debug)]
pub struct TimerInterval {
    count: u32,
}

impl TimerInterval {
    /// The count of the interval that triggered this event. Starts from `1`.
    pub fn count(&self) -> u32 {
        self.count
    }
}

/// A timer [`Event`] that is triggered when an [`ObservableTimer`] finishes, or is cancelled.
#[derive(Event, Debug)]
pub struct TimerFinished {
    cancelled: bool,
}

impl TimerFinished {
    /// `true` when the timer was manually cancelled before finishing.
    pub fn cancelled(&self) -> bool {
        self.cancelled
    }
}

fn on_timer_removed(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    let timer = world.get::<ObservableTimer>(entity).unwrap();
    if !timer.is_done() {
        world
            .commands()
            .trigger_targets(TimerFinished { cancelled: true }, entity);
    }
}

fn update_observable_timers(
    time: Res<Time>,
    mut timers: Query<(Entity, &mut ObservableTimer)>,
    mut commands: Commands,
) {
    let delta = time.delta();
    for (entity, mut timer) in timers.iter_mut() {
        if timer.is_added() {
            commands.trigger_targets(TimerStarted { _inner: () }, entity)
        }

        // The current interval number
        let interval_num = timer.elapsed_intervals + 1;

        // Tick the timer forward
        timer.tick(delta);

        // Trigger an interval event for every interval we finished this `tick()`
        for count in interval_num..(interval_num + timer.intervals_this_tick) {
            commands.trigger_targets(TimerInterval { count }, entity)
        }

        if timer.just_finished() {
            commands.trigger_targets(TimerFinished { cancelled: false }, entity);
            match timer.finish_behavior {
                TimerFinishBehavior::None => {}
                TimerFinishBehavior::RemoveComponent => {
                    commands.entity(entity).remove::<ObservableTimer>();
                }
                TimerFinishBehavior::DespawnEntity => {
                    commands.entity(entity).despawn();
                }
                TimerFinishBehavior::DespawnRecursive => {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}
