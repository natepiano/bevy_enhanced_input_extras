use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

/// Generates a BEI `InputAction` struct.
///
/// ```rust
/// action!(CameraHome);
/// ```
///
/// Expands to:
/// ```rust
/// #[derive(InputAction)]
/// #[action_output(bool)]
/// pub struct CameraHome;
/// ```
#[macro_export]
macro_rules! action {
    ($(#[$meta:meta])* $action:ident) => {
        $(#[$meta])*
        #[derive(InputAction)]
        #[action_output(bool)]
        pub struct $action;
    };
}

/// Generates a Bevy `Event` struct for BRP-triggerable events.
///
/// Unit event form:
/// ```rust
/// event!(PauseEvent);
/// ```
///
/// Payload event form:
/// ```rust
/// event!(ZoomToTarget { entity: Entity });
/// ```
///
/// Expands to either a unit struct with `Default`, or a named-field struct
/// without `Default` when payload fields are provided.
#[macro_export]
macro_rules! event {
    ($(#[$meta:meta])* $event:ident) => {
        $(#[$meta])*
        #[derive(Event, Reflect, Default)]
        #[reflect(Event)]
        pub struct $event;
    };
    ($(#[$meta:meta])* $event:ident { $($field:ident : $ty:ty),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Event, Reflect)]
        #[reflect(Event)]
        pub struct $event {
            $(pub $field: $ty,)+
        }
    };
}

/// Wires an input action to a command function through an intermediate event.
///
/// Registers two observers:
/// 1. `On<Start<Action>>` → triggers `Event`
/// 2. `On<Event>` → runs `command` via `run_system_cached`
///
/// The intermediate event decouples the keyboard input from the command execution.
/// This means the same command can be invoked both by a user-initiated keybinding
/// and programmatically (e.g. via `commands.trigger(PauseEvent)` or the Bevy Remote
/// Protocol's `world.trigger_event`), with both paths calling the same
/// `run_system_cached` command.
///
/// Use with `action!` and `event!` to generate the action and event structs.
///
/// Requires `bevy::prelude::*` in scope at the call site.
///
/// ```rust
/// bind_action_system!(app, PauseToggle, PauseEvent, pause_command);
/// ```
#[macro_export]
macro_rules! bind_action_system {
    ($app:expr, $action:ty, $event:ty, $command:path) => {
        $app.add_observer(
            |_: On<bevy_enhanced_input::action::events::Start<$action>>, mut commands: Commands| {
                commands.trigger(<$event>::default());
            },
        )
        .add_observer(|_: On<$event>, mut commands: Commands| {
            commands.run_system_cached($command);
        })
    };
}

/// Non-consuming modifier action for Cmd (macOS) / Ctrl (other platforms).
#[derive(InputAction)]
#[action_output(bool)]
struct PrimaryShortcutsModifier;

/// Non-consuming modifier action for Option (macOS) / Alt (other platforms).
#[derive(InputAction)]
#[action_output(bool)]
struct AltModifier;

/// Non-consuming modifier action for Ctrl on macOS (distinct from Cmd).
#[derive(InputAction)]
#[action_output(bool)]
struct ControlModifier;

/// Holds modifier entity IDs and automatically applies `BlockBy` to every
/// binding, preventing single-key actions from firing when any modifier is held.
///
/// # Modifier tracking by platform
///
/// **macOS:**
/// - `PrimaryShortcutsModifier` = Cmd (Super) — platform shortcuts (Cmd+S, Cmd+A)
/// - `ControlModifier` = Ctrl — separate physical key on Mac, blocks single keys
/// - `AltModifier` = Option — blocks single keys
/// - Both Cmd and Ctrl are in `all_modifiers`, so Ctrl+C and Cmd+C both block
///
/// **Windows / Linux:**
/// - `PrimaryShortcutsModifier` = Ctrl — platform shortcuts (Ctrl+S, Ctrl+A) AND blocks single keys
///   (tracked once, not duplicated)
/// - `AltModifier` = Alt — blocks single keys
/// - `ControlModifier` is not spawned (Ctrl is already `PrimaryShortcutsModifier`)
///
/// **All platforms:**
/// - Shift blocks single-key actions but not shift-key combos
/// - Alt/Option blocks single-key actions
/// - Platform shortcuts (`spawn_platform_key`) have no `BlockBy` since the modifier key itself
///   disambiguates
pub struct Keybindings<C: Component> {
    all_modifiers:       Vec<Entity>,
    non_shift_modifiers: Vec<Entity>,
    settings:            ActionSettings,
    _marker:             std::marker::PhantomData<C>,
}

impl<C: Component> Keybindings<C> {
    /// Spawns modifier actions and returns a `Keybindings` ready for use.
    ///
    /// The `S` type parameter is the `InputAction` to use for the shift modifier.
    /// This allows the caller to query for `Action<S>` to check shift state
    /// (e.g. for shift-click selection).
    pub fn new<S: InputAction>(spawner: &mut ActionSpawner<C>, settings: ActionSettings) -> Self {
        let non_consuming_modifier = ActionSettings {
            consume_input: false,
            require_reset: true,
            ..default()
        };
        let primary_modifier_bindings = if cfg!(target_os = "macos") {
            bindings![KeyCode::SuperLeft, KeyCode::SuperRight]
        } else {
            bindings![KeyCode::ControlLeft, KeyCode::ControlRight]
        };

        let shift = spawner
            .spawn((
                Action::<S>::new(),
                non_consuming_modifier,
                bindings![KeyCode::ShiftLeft, KeyCode::ShiftRight],
            ))
            .id();
        let primary = spawner
            .spawn((
                Action::<PrimaryShortcutsModifier>::new(),
                non_consuming_modifier,
                primary_modifier_bindings,
            ))
            .id();
        let alt = spawner
            .spawn((
                Action::<AltModifier>::new(),
                non_consuming_modifier,
                bindings![KeyCode::AltLeft, KeyCode::AltRight],
            ))
            .id();

        let mut all_modifiers = vec![shift, primary, alt];
        let mut non_shift_modifiers = vec![primary, alt];

        // On macOS, Ctrl is a separate physical key from Cmd — block it too.
        if cfg!(target_os = "macos") {
            let ctrl = spawner
                .spawn((
                    Action::<ControlModifier>::new(),
                    non_consuming_modifier,
                    bindings![KeyCode::ControlLeft, KeyCode::ControlRight],
                ))
                .id();
            all_modifiers.push(ctrl);
            non_shift_modifiers.push(ctrl);
        }

        Self {
            all_modifiers,
            non_shift_modifiers,
            settings,
            _marker: std::marker::PhantomData,
        }
    }

    /// Spawn an action bound to a single key, blocked by all modifiers.
    pub fn spawn_key<A: InputAction>(&self, spawner: &mut ActionSpawner<C>, key: KeyCode) {
        spawner.spawn((
            Action::<A>::new(),
            self.settings,
            BlockBy::new(self.all_modifiers.clone()),
            bindings![key],
        ));
    }

    /// Spawn an action bound to Shift + key, blocked by non-shift modifiers only.
    pub fn spawn_shift_key<A: InputAction>(&self, spawner: &mut ActionSpawner<C>, key: KeyCode) {
        spawner.spawn((
            Action::<A>::new(),
            self.settings,
            BlockBy::new(self.non_shift_modifiers.clone()),
            bindings![key.with_mod_keys(ModKeys::SHIFT)],
        ));
    }

    /// Spawn an action with arbitrary bindings, blocked by all modifiers.
    pub fn spawn_binding<A: InputAction, B: Bundle>(
        &self,
        spawner: &mut ActionSpawner<C>,
        bindings: B,
    ) {
        spawner.spawn((
            Action::<A>::new(),
            self.settings,
            BlockBy::new(self.all_modifiers.clone()),
            bindings,
        ));
    }

    /// Spawn an action with platform Cmd/Ctrl modifier. No `BlockBy` needed
    /// since the modifier key itself is the disambiguator.
    pub fn spawn_platform_key<A: InputAction>(&self, spawner: &mut ActionSpawner<C>, key: KeyCode) {
        let platform_bindings = if cfg!(target_os = "macos") {
            bindings![key.with_mod_keys(ModKeys::SUPER)]
        } else {
            bindings![key.with_mod_keys(ModKeys::CONTROL)]
        };
        spawner.spawn((Action::<A>::new(), self.settings, platform_bindings));
    }
}
