use super::input_events::{Key, MouseButton};
use crate::core::id::{Id, IdAllocator};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct ActionTag;

#[derive(Debug)]
pub struct GroupTag;

pub type ActionId = Id<ActionTag>;
pub type GroupId = Id<GroupTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    Key(Key),
    MouseButton(MouseButton),
}

#[derive(Debug, Clone)]
pub enum Binding {
    Trigger(Trigger),
    AnyOf(Vec<Binding>),
    AllOf(Vec<Binding>),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ActionState {
    pub down: bool,
    pub just_pressed: bool,
    pub just_released: bool,
    pub last_pressed: Option<Instant>,
    pub last_released: Option<Instant>,
}

#[derive(Debug, Default, Clone)]
pub struct ActionMap {
    action_ids: IdAllocator,
    name_to_id: HashMap<String, ActionId>,
    bindings: HashMap<ActionId, Binding>,
    states: HashMap<ActionId, ActionState>,

    group_ids: IdAllocator,
    group_name_to_id: HashMap<String, GroupId>,
    // group -> (action -> priority)
    group_members: HashMap<GroupId, HashMap<ActionId, i32>>,
}

impl ActionMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get (or create) an ActionId from a human-friendly name.
    pub fn action(&mut self, name: &str) -> ActionId {
        if let Some(id) = self.name_to_id.get(name) {
            return *id;
        }
        let id = self.action_ids.alloc::<ActionTag>();
        self.name_to_id.insert(name.to_string(), id);
        self.states.entry(id).or_default();
        id
    }

    pub fn bind(&mut self, id: ActionId, binding: Binding) {
        self.bindings.insert(id, binding);
        self.states.entry(id).or_default();
    }

    /// Get (or create) a GroupId from a human-friendly name.
    pub fn group(&mut self, name: &str) -> GroupId {
        if let Some(id) = self.group_name_to_id.get(name) {
            return *id;
        }
        let id = self.group_ids.alloc::<GroupTag>();
        self.group_name_to_id.insert(name.to_string(), id);
        self.group_members.entry(id).or_default();
        id
    }

    /// Set an action's priority inside a group. Actions can belong to multiple groups.
    pub fn set_group_priority(&mut self, group: GroupId, action: ActionId, priority: i32) {
        self.group_members
            .entry(group)
            .or_default()
            .insert(action, priority);
        self.states.entry(action).or_default();
    }

    /// Remove an action from a group.
    pub fn remove_from_group(&mut self, group: GroupId, action: ActionId) {
        if let Some(members) = self.group_members.get_mut(&group) {
            members.remove(&action);
        }
    }

    /// Returns all currently active actions in the group at the highest priority.
    ///
    /// - If none are active: returns empty.
    /// - If multiple tie at max priority: returns all tied actions.
    ///
    /// Returned list is deterministic (sorted by ActionId).
    pub fn active_in_group(&self, group: GroupId) -> Vec<ActionId> {
        let Some(members) = self.group_members.get(&group) else {
            return Vec::new();
        };

        let mut max_prio: Option<i32> = None;
        let mut winners: Vec<ActionId> = Vec::new();

        for (&action, &prio) in members {
            if !self.down(action) {
                continue;
            }
            match max_prio {
                None => {
                    max_prio = Some(prio);
                    winners.clear();
                    winners.push(action);
                }
                Some(cur) if prio > cur => {
                    max_prio = Some(prio);
                    winners.clear();
                    winners.push(action);
                }
                Some(cur) if prio == cur => {
                    winners.push(action);
                }
                _ => {}
            }
        }

        winners.sort();
        winners
    }

    pub fn state(&self, id: ActionId) -> ActionState {
        self.states.get(&id).copied().unwrap_or_default()
    }

    pub fn down(&self, id: ActionId) -> bool {
        self.state(id).down
    }

    pub fn just_pressed(&self, id: ActionId) -> bool {
        self.state(id).just_pressed
    }

    pub fn just_released(&self, id: ActionId) -> bool {
        self.state(id).just_released
    }

    pub fn was_pressed_within(&self, id: ActionId, within: Duration) -> bool {
        self.states
            .get(&id)
            .and_then(|s| s.last_pressed)
            .is_some_and(|t| t.elapsed() <= within)
    }

    pub(crate) fn update(&mut self, snapshot: &InputSnapshot<'_>) {
        let now = Instant::now();

        // Clear one-frame flags
        for state in self.states.values_mut() {
            state.just_pressed = false;
            state.just_released = false;
        }

        // Evaluate bindings
        for (&id, binding) in &self.bindings {
            let down_now = eval_binding(binding, snapshot);
            let state = self.states.entry(id).or_default();
            let down_prev = state.down;

            state.down = down_now;
            if down_now && !down_prev {
                state.just_pressed = true;
                state.last_pressed = Some(now);
            } else if !down_now && down_prev {
                state.just_released = true;
                state.last_released = Some(now);
            }
        }
    }
}

pub(crate) struct InputSnapshot<'a> {
    pub pressed_keys: &'a HashSet<Key>,
    pub pressed_buttons: &'a HashSet<MouseButton>,
}

fn eval_binding(binding: &Binding, snapshot: &InputSnapshot<'_>) -> bool {
    match binding {
        Binding::Trigger(t) => match t {
            Trigger::Key(k) => snapshot.pressed_keys.contains(k),
            Trigger::MouseButton(b) => snapshot.pressed_buttons.contains(b),
        },
        Binding::AnyOf(list) => list.iter().any(|b| eval_binding(b, snapshot)),
        Binding::AllOf(list) => !list.is_empty() && list.iter().all(|b| eval_binding(b, snapshot)),
    }
}
