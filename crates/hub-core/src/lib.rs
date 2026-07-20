//! View- and transport-independent domain logic for Control Deck.
//!
//! This crate intentionally depends only on the Rust standard library. The
//! server, UI, Windows adapter, and persistence crates sit outside this
//! boundary and consume the public API exposed here.

use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Protocol error codes fixed by design decision D3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidToken,
    MalformedMessage,
    CommandNotAllowed,
    RevisionConflict,
    AdapterFailure,
    InternalError,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidToken => "INVALID_TOKEN",
            Self::MalformedMessage => "MALFORMED_MESSAGE",
            Self::CommandNotAllowed => "COMMAND_NOT_ALLOWED",
            Self::RevisionConflict => "REVISION_CONFLICT",
            Self::AdapterFailure => "ADAPTER_FAILURE",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenState {
    Idle,
    Editor,
}

impl ScreenState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Editor => "editor",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseState {
    Editing,
    Saved,
}

impl PhaseState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Editing => "editing",
            Self::Saved => "saved",
        }
    }
}

/// The two state dimensions are intentionally separate. Neither setter has
/// access to, nor changes, the other dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateSnapshot {
    pub screen_state: ScreenState,
    pub phase_state: PhaseState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HubState {
    screen_state: ScreenState,
    phase_state: PhaseState,
}

impl Default for HubState {
    fn default() -> Self {
        Self {
            screen_state: ScreenState::Idle,
            phase_state: PhaseState::Editing,
        }
    }
}

impl HubState {
    pub fn new(screen_state: ScreenState, phase_state: PhaseState) -> Self {
        Self {
            screen_state,
            phase_state,
        }
    }

    pub fn snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            screen_state: self.screen_state,
            phase_state: self.phase_state,
        }
    }

    pub fn set_screen_state(&mut self, state: ScreenState) {
        self.screen_state = state;
    }

    pub fn set_phase_state(&mut self, state: PhaseState) {
        self.phase_state = state;
    }
}

/// Profile-derived command allow-list. Command IDs are opaque semantic IDs;
/// this type never interprets them as shell commands or key strings.
#[derive(Debug, Clone, Default)]
pub struct CommandRegistry {
    allowed: BTreeSet<String>,
}

impl CommandRegistry {
    pub fn new<I, S>(command_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            allowed: command_ids.into_iter().map(Into::into).collect(),
        }
    }

    pub fn is_allowed(&self, command_id: &str) -> bool {
        self.allowed.contains(command_id)
    }

    pub fn allowed_command_ids(&self) -> impl Iterator<Item = &str> {
        self.allowed.iter().map(String::as_str)
    }

    pub fn replace<I, S>(&mut self, command_ids: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed = command_ids.into_iter().map(Into::into).collect();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInvocation {
    pub request_id: String,
    pub command_id: String,
    pub args: BTreeMap<String, String>,
}

impl CommandInvocation {
    pub fn new(request_id: impl Into<String>, command_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            command_id: command_id.into(),
            args: BTreeMap::new(),
        }
    }
}

/// Boundary implemented by adapter crates. Domain code supplies an allowed,
/// non-duplicate invocation and receives a protocol-safe result.
pub trait CommandExecutor {
    fn execute(&mut self, invocation: &CommandInvocation) -> Result<String, HubError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubError {
    pub code: ErrorCode,
    pub message: String,
}

impl HubError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatus {
    Success,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    pub status: CommandStatus,
    pub error_code: Option<ErrorCode>,
    pub message: String,
}

impl CommandResult {
    fn success(message: String) -> Self {
        Self {
            status: CommandStatus::Success,
            error_code: None,
            message,
        }
    }

    fn rejected(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status: CommandStatus::Rejected,
            error_code: Some(code),
            message: message.into(),
        }
    }

    fn failed(error: HubError) -> Self {
        Self {
            status: CommandStatus::Failed,
            error_code: Some(error.code),
            message: error.message,
        }
    }
}

/// `executed` is false for both rejected messages and replayed results.
/// `replayed` distinguishes a duplicate request from an initial rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutcome {
    pub result: CommandResult,
    pub executed: bool,
    pub replayed: bool,
}

#[derive(Debug, Clone)]
pub struct CommandService {
    registry: CommandRegistry,
    completed_requests: HashMap<String, CommandResult>,
}

impl CommandService {
    pub fn new(registry: CommandRegistry) -> Self {
        Self {
            registry,
            completed_requests: HashMap::new(),
        }
    }

    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    pub fn replace_registry(&mut self, registry: CommandRegistry) {
        self.registry = registry;
    }

    pub fn completed_request_count(&self) -> usize {
        self.completed_requests.len()
    }

    /// Validate, execute at most once, and cache the result by request ID.
    /// Re-sending the same ID returns the original result without reaching
    /// the adapter, including when the original result was a rejection or
    /// adapter failure.
    pub fn invoke<E: CommandExecutor>(
        &mut self,
        invocation: CommandInvocation,
        executor: &mut E,
    ) -> CommandOutcome {
        if let Some(result) = self.completed_requests.get(&invocation.request_id) {
            return CommandOutcome {
                result: result.clone(),
                executed: false,
                replayed: true,
            };
        }

        let (result, executed) =
            if invocation.request_id.trim().is_empty() || invocation.command_id.trim().is_empty() {
                (
                    CommandResult::rejected(
                        ErrorCode::MalformedMessage,
                        "requestId and commandId must not be empty",
                    ),
                    false,
                )
            } else if !self.registry.is_allowed(&invocation.command_id) {
                (
                    CommandResult::rejected(
                        ErrorCode::CommandNotAllowed,
                        format!("command is not allowed: {}", invocation.command_id),
                    ),
                    false,
                )
            } else {
                let result = match executor.execute(&invocation) {
                    Ok(message) => CommandResult::success(message),
                    Err(error) => CommandResult::failed(error),
                };
                (result, true)
            };

        self.completed_requests
            .insert(invocation.request_id, result.clone());

        CommandOutcome {
            result,
            executed,
            replayed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorValue {
    Text(String),
    Number(f64),
    Toggle(bool),
    Select(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorConstraint {
    Text {
        min_length: Option<usize>,
        max_length: Option<usize>,
    },
    Number {
        min: Option<f64>,
        max: Option<f64>,
    },
    Slider {
        min: f64,
        max: f64,
        step: f64,
    },
    Toggle,
    Select {
        options: BTreeSet<String>,
    },
}

impl InspectorConstraint {
    fn validate(&self, value: &InspectorValue) -> Result<(), HubError> {
        let valid = match (self, value) {
            (
                Self::Text {
                    min_length,
                    max_length,
                },
                InspectorValue::Text(value),
            ) => {
                let length = value.chars().count();
                min_length.is_none_or(|min| length >= min)
                    && max_length.is_none_or(|max| length <= max)
            }
            (Self::Number { min, max }, InspectorValue::Number(value)) => {
                value.is_finite()
                    && min.is_none_or(|min| *value >= min)
                    && max.is_none_or(|max| *value <= max)
            }
            (Self::Slider { min, max, step }, InspectorValue::Number(value)) => {
                if !value.is_finite() || *step <= 0.0 || *value < *min || *value > *max {
                    false
                } else {
                    let steps = (*value - *min) / *step;
                    (steps - steps.round()).abs() <= 1e-9
                }
            }
            (Self::Toggle, InspectorValue::Toggle(_)) => true,
            (Self::Select { options }, InspectorValue::Select(value)) => options.contains(value),
            _ => false,
        };

        if valid {
            Ok(())
        } else {
            Err(HubError::new(
                ErrorCode::MalformedMessage,
                "inspector value does not satisfy its schema",
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorField {
    constraint: InspectorConstraint,
    value: InspectorValue,
}

impl InspectorField {
    pub fn new(constraint: InspectorConstraint, value: InspectorValue) -> Result<Self, HubError> {
        constraint.validate(&value)?;
        Ok(Self { constraint, value })
    }

    pub fn value(&self) -> &InspectorValue {
        &self.value
    }

    pub fn constraint(&self) -> &InspectorConstraint {
        &self.constraint
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorTarget {
    target_id: String,
    revision: u64,
    fields: BTreeMap<String, InspectorField>,
}

impl InspectorTarget {
    pub fn new(target_id: impl Into<String>, revision: u64) -> Self {
        Self {
            target_id: target_id.into(),
            revision,
            fields: BTreeMap::new(),
        }
    }

    pub fn target_id(&self) -> &str {
        &self.target_id
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn fields(&self) -> &BTreeMap<String, InspectorField> {
        &self.fields
    }

    pub fn insert_field(
        &mut self,
        path: impl Into<String>,
        field: InspectorField,
    ) -> Option<InspectorField> {
        self.fields.insert(path.into(), field)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedInspectorValue {
    pub target_id: String,
    pub path: String,
    pub value: InspectorValue,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorUpdateError {
    pub error: HubError,
    /// Included on a revision conflict so clients can immediately converge.
    pub latest: Option<ConfirmedInspectorValue>,
}

#[derive(Debug, Clone, Default)]
pub struct InspectorStore {
    targets: BTreeMap<String, InspectorTarget>,
}

impl InspectorStore {
    pub fn insert_target(&mut self, target: InspectorTarget) -> Option<InspectorTarget> {
        self.targets.insert(target.target_id.clone(), target)
    }

    pub fn target(&self, target_id: &str) -> Option<&InspectorTarget> {
        self.targets.get(target_id)
    }

    /// Apply an update if its revision matches. A successful mutation advances
    /// the target revision exactly once. A rejected mutation changes nothing.
    pub fn update(
        &mut self,
        target_id: &str,
        path: &str,
        value: InspectorValue,
        expected_revision: u64,
    ) -> Result<ConfirmedInspectorValue, InspectorUpdateError> {
        let target = self
            .targets
            .get_mut(target_id)
            .ok_or_else(|| InspectorUpdateError {
                error: HubError::new(
                    ErrorCode::MalformedMessage,
                    format!("unknown inspector target: {target_id}"),
                ),
                latest: None,
            })?;

        let field = target
            .fields
            .get_mut(path)
            .ok_or_else(|| InspectorUpdateError {
                error: HubError::new(
                    ErrorCode::MalformedMessage,
                    format!("unknown inspector path: {path}"),
                ),
                latest: None,
            })?;

        if expected_revision != target.revision {
            return Err(InspectorUpdateError {
                error: HubError::new(
                    ErrorCode::RevisionConflict,
                    format!(
                        "expected revision {expected_revision}, current revision is {}",
                        target.revision
                    ),
                ),
                latest: Some(ConfirmedInspectorValue {
                    target_id: target_id.to_owned(),
                    path: path.to_owned(),
                    value: field.value.clone(),
                    revision: target.revision,
                }),
            });
        }

        field
            .constraint
            .validate(&value)
            .map_err(|error| InspectorUpdateError {
                error,
                latest: None,
            })?;

        // Determine the next revision before mutating the field so even the
        // overflow edge case remains atomic.
        let next_revision = target
            .revision
            .checked_add(1)
            .ok_or_else(|| InspectorUpdateError {
                error: HubError::new(ErrorCode::InternalError, "revision overflow"),
                latest: None,
            })?;
        field.value = value.clone();
        target.revision = next_revision;

        Ok(ConfirmedInspectorValue {
            target_id: target_id.to_owned(),
            path: path.to_owned(),
            value,
            revision: target.revision,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct CountingExecutor {
        calls: usize,
    }

    impl CommandExecutor for CountingExecutor {
        fn execute(&mut self, invocation: &CommandInvocation) -> Result<String, HubError> {
            self.calls += 1;
            Ok(format!("{} completed", invocation.command_id))
        }
    }

    #[test]
    fn command_not_in_allow_list_is_rejected() {
        let registry = CommandRegistry::new(["editor.save"]);
        let mut service = CommandService::new(registry);
        let mut executor = CountingExecutor::default();

        let outcome = service.invoke(
            CommandInvocation::new("req_disallowed", "shell.run"),
            &mut executor,
        );

        assert_eq!(outcome.result.status, CommandStatus::Rejected);
        assert_eq!(
            outcome.result.error_code,
            Some(ErrorCode::CommandNotAllowed)
        );
        assert_eq!(
            outcome.result.error_code.unwrap().as_str(),
            "COMMAND_NOT_ALLOWED"
        );
        assert!(!outcome.executed);
        assert!(!outcome.replayed);
        assert_eq!(executor.calls, 0);
    }

    #[test]
    fn duplicate_request_id_replays_result_without_second_execution() {
        let registry = CommandRegistry::new(["editor.save"]);
        let mut service = CommandService::new(registry);
        let mut executor = CountingExecutor::default();
        let invocation = CommandInvocation::new("req_001", "editor.save");

        let first = service.invoke(invocation.clone(), &mut executor);
        let duplicate = service.invoke(invocation, &mut executor);

        assert_eq!(first.result.status, CommandStatus::Success);
        assert!(first.executed);
        assert!(!first.replayed);
        assert_eq!(duplicate.result, first.result);
        assert!(!duplicate.executed);
        assert!(duplicate.replayed);
        assert_eq!(executor.calls, 1);
        assert_eq!(service.completed_request_count(), 1);
    }

    fn inspector_with_hp() -> InspectorStore {
        let mut target = InspectorTarget::new("char_knight", 12);
        target.insert_field(
            "stats.hp",
            InspectorField::new(
                InspectorConstraint::Slider {
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                },
                InspectorValue::Number(35.0),
            )
            .unwrap(),
        );
        let mut store = InspectorStore::default();
        store.insert_target(target);
        store
    }

    #[test]
    fn revision_conflict_returns_latest_value_and_does_not_mutate() {
        let mut store = inspector_with_hp();

        let conflict = store
            .update("char_knight", "stats.hp", InspectorValue::Number(40.0), 11)
            .unwrap_err();

        assert_eq!(conflict.error.code, ErrorCode::RevisionConflict);
        assert_eq!(conflict.error.code.as_str(), "REVISION_CONFLICT");
        let latest = conflict.latest.expect("conflicts include latest state");
        assert_eq!(latest.revision, 12);
        assert_eq!(latest.value, InspectorValue::Number(35.0));
        assert_eq!(store.target("char_knight").unwrap().revision(), 12);
    }

    #[test]
    fn accepted_inspector_update_increments_revision_once() {
        let mut store = inspector_with_hp();

        let confirmed = store
            .update("char_knight", "stats.hp", InspectorValue::Number(40.0), 12)
            .unwrap();

        assert_eq!(confirmed.value, InspectorValue::Number(40.0));
        assert_eq!(confirmed.revision, 13);
        assert_eq!(store.target("char_knight").unwrap().revision(), 13);
    }

    #[test]
    fn inspector_schema_rejects_out_of_range_value_without_revision_change() {
        let mut store = inspector_with_hp();

        let rejected = store
            .update("char_knight", "stats.hp", InspectorValue::Number(101.0), 12)
            .unwrap_err();

        assert_eq!(rejected.error.code, ErrorCode::MalformedMessage);
        assert_eq!(store.target("char_knight").unwrap().revision(), 12);
    }

    #[test]
    fn revision_overflow_does_not_partially_mutate_value() {
        let mut target = InspectorTarget::new("char_knight", u64::MAX);
        target.insert_field(
            "stats.hp",
            InspectorField::new(
                InspectorConstraint::Number {
                    min: Some(0.0),
                    max: Some(100.0),
                },
                InspectorValue::Number(35.0),
            )
            .unwrap(),
        );
        let mut store = InspectorStore::default();
        store.insert_target(target);

        let rejected = store
            .update(
                "char_knight",
                "stats.hp",
                InspectorValue::Number(40.0),
                u64::MAX,
            )
            .unwrap_err();

        assert_eq!(rejected.error.code, ErrorCode::InternalError);
        let target = store.target("char_knight").unwrap();
        assert_eq!(target.revision(), u64::MAX);
        assert_eq!(
            target.fields()["stats.hp"].value(),
            &InspectorValue::Number(35.0)
        );
    }

    #[test]
    fn screen_and_phase_state_update_independently() {
        let mut state = HubState::default();

        state.set_screen_state(ScreenState::Editor);
        assert_eq!(
            state.snapshot(),
            StateSnapshot {
                screen_state: ScreenState::Editor,
                phase_state: PhaseState::Editing,
            }
        );

        state.set_phase_state(PhaseState::Saved);
        assert_eq!(
            state.snapshot(),
            StateSnapshot {
                screen_state: ScreenState::Editor,
                phase_state: PhaseState::Saved,
            }
        );
    }
}
