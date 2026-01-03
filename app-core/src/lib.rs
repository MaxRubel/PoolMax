pub mod message_queue;
pub mod structs;

use crate::{
  message_queue::MessageQueue,
  structs::{Filter, Heater, Lights, Mech, PoolOrSpa, PoolValve, PrevState, System},
};

impl<M: Mech, L: Lights> System<M, L> {
  pub fn new(mech: M, lights: L) -> Self {
    System {
      main_valve_orientation: PoolOrSpa::Pool,
      pool_valve_orientation: PoolValve::Skimmer,
      filter: Filter {
        running_schedule: false,
        quick_clean: false,
      },
      heater: Heater {
        mode: PoolOrSpa::Pool,
        on: false,
      },
      jets_on: false,
      errors: [None; 10],
      in_progress: false,
      mech,
      lights,
      internal_test: false,
      prev_state: None,
      message_queue: MessageQueue::new(),
    }
  }

  // Jets
  pub fn toggle_jets(&mut self) -> bool {
    if self.jets_on {
      log_msg!(self.message_queue, "Jets OFF");
    } else {
      log_msg!(self.message_queue, "Jets ON");
    }

    let new = !self.jets_on;

    self.mech.jets_on_toggle(new);
    self.lights.jets_on(new);
    self.jets_on = new;

    true
  }

  // Filter
  pub fn filter_delay(&mut self) -> bool {
    log_msg!(self.message_queue, "Running: Filter ON");

    self.with_progress_light(|s| {
      if !s.internal_test {
        s.mech.delay_secs(10);
      }
    });

    log_msg!(self.message_queue, "Finish: Filter primed and running");
    true
  }

  pub fn toggle_quick_clean(&mut self) -> bool {
    if self.filter.quick_clean {
      log_msg!(self.message_queue, "Quick clean OFF");
      self.lights.quick_clean(false);
      self.mech.set_quick_clean(false);
      self.filter.quick_clean = false;
    } else {
      log_msg!(self.message_queue, "Quick Clean ON");
      self.start_quick_clean();
    }
    true
  }

  pub fn toggle_filter_schedule(&mut self) -> bool {
    let new = !self.filter.running_schedule;
    self.lights.filter_schedule(new);
    self.filter.running_schedule = new;

    if new {
      log_msg!(self.message_queue, "-Protect-");
      log_msg!(self.message_queue, "Filter schedule is ON");
      self.filter_delay();
    } else {
      log_msg!(self.message_queue, "Filter schedule is OFF")
    }

    true
  }

  pub fn stop_filter(&mut self) -> bool {
    if !self.filter.quick_clean && !self.filter.running_schedule {
      return false;
    }
    log_msg!(self.message_queue, "Running: Turning filter OFF");

    self.with_progress_light(|s| {
      s.mech.set_quick_clean(false);
      s.mech.mech_set_filter_sched(false);

      if !s.internal_test {
        s.mech.delay_secs(5);
      }

      s.filter.quick_clean = false;
      s.filter.running_schedule = false;
    });

    log_msg!(self.message_queue, "Finish: Filter is OFF");

    true
  }

  pub fn start_quick_clean(&mut self) -> bool {
    if self.filter.quick_clean {
      return false;
    }

    log_msg!(self.message_queue, "Running: Turning quick clean ON");

    self.lights.quick_clean(true);
    self.mech.set_quick_clean(true);
    self.filter_delay();
    self.filter.quick_clean = true;

    log_msg!(self.message_queue, "Complete: Quick clean is ON");

    true
  }

  // Heater
  pub fn set_heater_on(&mut self, b: bool) -> bool {
    if self.heater.on == b {
      return false;
    }

    self.mech.heater_on_toggle(b);
    self.lights.heater_on(b);
    self.heater.on = b;

    if b {
      log_msg!(self.message_queue, "Heater ON");
    } else {
      log_msg!(self.message_queue, "Heater OFF");
    }

    true
  }

  pub fn toggle_heater_on(&mut self) {
    if self.heater.on {
      self.set_heater_on(false);
    } else {
      self.set_heater_on(true);
    }
  }

  pub fn toggle_heat_mode(&mut self) -> bool {
    if self.heater.mode == PoolOrSpa::Pool {
      self.set_heat_mode(PoolOrSpa::Spa);
    } else {
      self.set_heat_mode(PoolOrSpa::Pool);
    }

    true
  }

  pub fn set_heat_mode(&mut self, m: PoolOrSpa) -> bool {
    if self.heater.mode == m {
      return false;
    }

    log_msg!(self.message_queue, "Heat mode set to {}", m);
    self.heater.mode = m;
    self.lights.heater_mode(m);
    self.mech.heater_mode_toggle(m);
    return true;
  }

  // Valves
  pub fn set_main_valves(&mut self, m: PoolOrSpa) -> bool {
    if self.main_valve_orientation == m {
      return false;
    }

    self.lights.main_valve_orientation(m);
    log_msg!(
      self.message_queue,
      "Start: Changing main valve orientation to {}",
      m
    );

    if self.filter.quick_clean || self.filter.running_schedule {
      log_msg!(self.message_queue, "-Protect- Turning filter OFF");
      self.stop_filter();
    }

    self.with_progress_light(|s| {
      s.mech.mech_main_valve_to(m);
      if !s.internal_test {
        s.mech.delay_secs(10);
      }

      s.main_valve_orientation = m;
      log_msg!(s.message_queue, "Finish: Valves changed to {} mode", m);
    });

    true
  }

  pub fn toggle_main_valves(&mut self) {
    let prev_state = Some(PrevState {
      heater: Heater {
        mode: self.heater.mode,
        on: self.heater.on,
      },
      filter: Filter {
        running_schedule: self.filter.running_schedule,
        quick_clean: self.filter.quick_clean,
      },
      main_valve_orientation: None,
    });

    match self.main_valve_orientation {
      PoolOrSpa::Pool => self.set_main_valves(PoolOrSpa::Spa),
      PoolOrSpa::Spa => self.set_main_valves(PoolOrSpa::Pool),
    };

    self.restore_previous_state(prev_state);
  }

  // Routines
  pub fn auto_spa(&mut self, ignore: Option<bool>) -> (bool, bool, bool) {
    let prev_state = Some(PrevState {
      heater: Heater {
        mode: self.heater.mode,
        on: self.heater.on,
      },
      filter: Filter {
        running_schedule: self.filter.running_schedule,
        quick_clean: self.filter.quick_clean,
      },
      main_valve_orientation: Some(self.main_valve_orientation),
    });

    log_msg!(self.message_queue, "Starting Spa... Enjoy! xo");

    self.set_main_valves(PoolOrSpa::Spa);

    let op2 = self.set_heater_on(true);
    let op1 = self.set_heat_mode(PoolOrSpa::Spa);
    let op3 = self.start_quick_clean();

    if !self.internal_test {
      self.mech.delay_secs(15);
    }

    log_msg!(self.message_queue, "Delay of 3 hours");

    if !ignore.unwrap_or(false) {
      self.restore_previous_state(prev_state);
    }

    log_msg!(self.message_queue, "Complete: Spa Mode");
    return (op1, op2, op3);
  }

  fn restore_previous_state(&mut self, o: Option<PrevState>) -> bool {
    log_msg!(self.message_queue, "Start: Restoring previous state");
    if let Some(n) = o {
      if n.filter.running_schedule != self.filter.running_schedule {
        self.toggle_filter_schedule();
      }

      if let Some(orientation) = n.main_valve_orientation {
        self.set_main_valves(orientation);
      }

      if n.filter.quick_clean {
        self.start_quick_clean();
      }

      self.set_heat_mode(n.heater.mode);
      self.set_heater_on(n.heater.on);
    }

    log_msg!(self.message_queue, "Finish: Previous state restored");

    true
  }

  pub fn get_next_message(&mut self) -> Option<&str> {
    self.message_queue.peek().map(|m| m.get_str())
  }

  pub fn pop_message(&mut self) -> Option<String> {
    self.message_queue.pop().map(|m| m.get_str().to_string())
  }

  // Lights
  fn with_progress_light<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self),
  {
    self.in_progress = true;
    self.lights.in_progress(true);
    f(self);
    self.lights.in_progress(false);
    self.in_progress = false;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::structs::HasOSLights;
  use crate::structs::HasOSMech;

  #[test]
  fn filter_does_not_start_if_running() {
    let mut sys = System::<HasOSMech, HasOSLights> {
      filter: Filter {
        running_schedule: false,
        quick_clean: true,
      },
      ..Default::default()
    };

    assert_eq!(sys.start_quick_clean(), false);
    assert_eq!(sys.filter.quick_clean, true);
  }

  #[test]
  fn filter_starts_if_not_running() {
    let mut sys = System::<HasOSMech, HasOSLights> {
      filter: Filter {
        running_schedule: false,
        quick_clean: false,
      },
      ..Default::default()
    };

    assert_eq!(sys.start_quick_clean(), true);
    assert_eq!(sys.filter.quick_clean, true);
  }

  #[test]
  fn filter_does_not_stop_if_stopped_already() {
    let mut sys = System::<HasOSMech, HasOSLights> {
      filter: Filter {
        running_schedule: false,
        quick_clean: false,
      },
      ..Default::default()
    };

    assert_eq!(sys.stop_filter(), false);
    assert_eq!(sys.filter.running_schedule, false);
    assert_eq!(sys.filter.quick_clean, false);
    assert_eq!(sys.stop_filter(), false);
    assert_eq!(sys.filter.running_schedule, false);
    assert_eq!(sys.filter.quick_clean, false);
  }

  #[test]
  fn heater_status_changes_correctly() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();

    sys.set_heater_on(true);
    assert_eq!(sys.heater.on, true);
    assert_eq!(sys.set_heater_on(true), false);
    assert_eq!(sys.heater.on, true);

    sys.set_heater_on(false);
    assert_eq!(sys.heater.on, false);
    assert_eq!(sys.set_heater_on(false), false);
    assert_eq!(sys.heater.on, false);
  }

  #[test]
  fn spa_mode_behaves_expected() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();
    assert_eq!(sys.auto_spa(Some(true)), (true, true, true));
    assert_eq!(sys.filter.quick_clean, true);
    assert_eq!(sys.heater.on, true);
    assert_eq!(sys.main_valve_orientation, PoolOrSpa::Spa);

    assert_eq!(sys.auto_spa(Some(true)), (false, false, false));

    // heater is already running for pool
    let mut sys = System::<HasOSMech, HasOSLights> {
      heater: Heater {
        mode: PoolOrSpa::Pool,
        on: true,
      },
      main_valve_orientation: PoolOrSpa::Pool,
      ..Default::default()
    };

    let result = sys.auto_spa(Some(true));
    assert_eq!(result, (true, false, true));

    println!("-------------------------");
    // heater is already running for spa
    let mut sys = System::<HasOSMech, HasOSLights> {
      heater: Heater {
        mode: PoolOrSpa::Spa,
        on: true,
      },
      main_valve_orientation: PoolOrSpa::Spa,
      ..Default::default()
    };

    let result = sys.auto_spa(Some(true));
    assert_eq!(result, (false, false, true));

    // already in spa mode:
    let mut sys = System::<HasOSMech, HasOSLights> {
      heater: Heater {
        mode: PoolOrSpa::Spa,
        on: true,
      },
      filter: Filter {
        running_schedule: false,
        quick_clean: true,
      },
      main_valve_orientation: PoolOrSpa::Spa,
      ..Default::default()
    };

    let result = sys.auto_spa(Some(true));
    assert_eq!(result, (false, false, false))
  }

  #[test]
  fn set_main_valves_is_safe() {
    let mut sys = System::<HasOSMech, HasOSLights> {
      filter: Filter {
        running_schedule: false,
        quick_clean: true,
      },
      main_valve_orientation: PoolOrSpa::Spa,
      ..Default::default()
    };

    let result = sys.set_main_valves(PoolOrSpa::Pool);
    assert_eq!(result, true);
  }

  #[test]
  fn set_main_valves_does_not_run_if_uneeded() {
    let mut sys = System::<HasOSMech, HasOSLights> {
      main_valve_orientation: PoolOrSpa::Pool,
      ..Default::default()
    };

    let result = sys.set_main_valves(PoolOrSpa::Pool);
    assert_eq!(result, false);
    assert_eq!(sys.main_valve_orientation, PoolOrSpa::Pool);
  }
  #[test]
  fn set_main_valves_runs_if_uneeded() {
    let mut sys = System::<HasOSMech, HasOSLights> {
      main_valve_orientation: PoolOrSpa::Pool,
      ..Default::default()
    };

    let result = sys.set_main_valves(PoolOrSpa::Spa);
    assert_eq!(result, true);
    assert_eq!(sys.main_valve_orientation, PoolOrSpa::Spa);
  }

  #[test]
  fn jets_behave_expected() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();

    assert_eq!(sys.jets_on, false);
    sys.toggle_jets();
    assert_eq!(sys.jets_on, true);
    sys.toggle_jets();
    assert_eq!(sys.jets_on, false);
  }

  #[test]
  fn spa_mode_doesnt_harm_filter() {
    let mut sys = System::<HasOSMech, HasOSLights> {
      filter: Filter {
        running_schedule: false,
        quick_clean: true,
      },
      ..Default::default()
    };

    sys.auto_spa(Some(true));

    let mut messages = Vec::new();
    while let Some(msg) = sys.pop_message() {
      messages.push(msg.to_string());
    }

    let has_valve_message = messages
      .iter()
      .any(|msg| msg.contains("-Protect- Turning filter OFF"));

    println!("{}", has_valve_message);
    assert!(has_valve_message, "Expected message not found");
  }

  #[test]
  fn filter_schedule_toggles_correctly() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();

    assert_eq!(sys.filter.running_schedule, false);
    sys.toggle_filter_schedule();
    assert_eq!(sys.filter.running_schedule, true);

    // Check for protection message and schedule ON message
    let mut messages = Vec::new();
    while let Some(msg) = sys.pop_message() {
      messages.push(msg);
    }
    assert!(messages.iter().any(|m| m.contains("-Protect-")));
    assert!(messages.iter().any(|m| m.contains("Filter schedule is ON")));

    sys.toggle_filter_schedule();
    assert_eq!(sys.filter.running_schedule, false);

    // Check for schedule OFF message
    let mut messages = Vec::new();
    while let Some(msg) = sys.pop_message() {
      messages.push(msg);
    }
    assert!(messages
      .iter()
      .any(|m| m.contains("Filter schedule is OFF")));
  }

  #[test]
  fn quick_clean_toggles_correctly() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();

    assert_eq!(sys.filter.quick_clean, false);
    sys.toggle_quick_clean();
    assert_eq!(sys.filter.quick_clean, true);

    // Verify ON message
    let mut messages = Vec::new();
    while let Some(msg) = sys.pop_message() {
      messages.push(msg);
    }
    assert!(messages.iter().any(|m| m.contains("Quick Clean ON")));

    sys.toggle_quick_clean();
    assert_eq!(sys.filter.quick_clean, false);

    // Verify OFF message
    let mut messages = Vec::new();
    while let Some(msg) = sys.pop_message() {
      messages.push(msg);
    }
    assert!(messages.iter().any(|m| m.contains("Quick clean OFF")));
  }

  #[test]
  fn heat_mode_toggles_correctly() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();

    assert_eq!(sys.heater.mode, PoolOrSpa::Pool);
    sys.toggle_heat_mode();
    assert_eq!(sys.heater.mode, PoolOrSpa::Spa);

    sys.toggle_heat_mode();
    assert_eq!(sys.heater.mode, PoolOrSpa::Pool);
  }

  #[test]
  fn heater_toggles_correctly() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();

    assert_eq!(sys.heater.on, false);
    sys.toggle_heater_on();
    assert_eq!(sys.heater.on, true);

    sys.toggle_heater_on();
    assert_eq!(sys.heater.on, false);
  }

  #[test]
  fn main_valves_toggle_and_restore_state() {
    let mut sys = System::<HasOSMech, HasOSLights> {
      heater: Heater {
        mode: PoolOrSpa::Pool,
        on: true,
      },
      filter: Filter {
        running_schedule: true,
        quick_clean: false,
      },
      main_valve_orientation: PoolOrSpa::Pool,
      ..Default::default()
    };

    sys.toggle_main_valves();

    assert_eq!(sys.main_valve_orientation, PoolOrSpa::Spa);
    assert_eq!(sys.heater.on, true);
    assert_eq!(sys.heater.mode, PoolOrSpa::Pool);
    assert_eq!(sys.filter.running_schedule, true);
  }

  // Restore previous state tests
  #[test]
  fn restore_previous_state_restores_spa_valve() {
    let mut sys = System::<HasOSMech, HasOSLights> {
      main_valve_orientation: PoolOrSpa::Spa,
      ..Default::default()
    };

    let prev_state = Some(PrevState {
      heater: Heater {
        mode: PoolOrSpa::Spa,
        on: false,
      },
      filter: Filter {
        running_schedule: false,
        quick_clean: false,
      },
      main_valve_orientation: Some(PoolOrSpa::Spa),
    });

    // Change to Pool
    sys.set_main_valves(PoolOrSpa::Pool);
    assert_eq!(sys.main_valve_orientation, PoolOrSpa::Pool);

    // Restore should bring back to Spa
    sys.restore_previous_state(prev_state);
    assert_eq!(sys.main_valve_orientation, PoolOrSpa::Spa);
  }

  #[test]
  fn restore_previous_state_restores_filter_schedule() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();

    let prev_state = Some(PrevState {
      heater: Heater {
        mode: PoolOrSpa::Pool,
        on: false,
      },
      filter: Filter {
        running_schedule: true,
        quick_clean: false,
      },
      main_valve_orientation: None,
    });

    assert_eq!(sys.filter.running_schedule, false);
    sys.restore_previous_state(prev_state);
    assert_eq!(sys.filter.running_schedule, true);
  }

  // Edge case tests
  #[test]
  fn auto_spa_with_explicit_false_restores_state() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();

    sys.auto_spa(Some(false));

    // Should restore to original state
    assert_eq!(sys.filter.quick_clean, false);
    assert_eq!(sys.heater.on, false);
    assert_eq!(sys.main_valve_orientation, PoolOrSpa::Pool);
  }

  #[test]
  fn auto_spa_with_none_restores_state() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();

    sys.auto_spa(None);

    // Should restore to original state (None means restore)
    assert_eq!(sys.filter.quick_clean, false);
    assert_eq!(sys.heater.on, false);
    assert_eq!(sys.main_valve_orientation, PoolOrSpa::Pool);
  }

  #[test]
  fn valve_change_stops_running_schedule() {
    let mut sys = System::<HasOSMech, HasOSLights> {
      filter: Filter {
        running_schedule: true,
        quick_clean: false,
      },
      main_valve_orientation: PoolOrSpa::Pool,
      ..Default::default()
    };

    sys.set_main_valves(PoolOrSpa::Spa);

    // Filter should be stopped for safety
    assert_eq!(sys.filter.running_schedule, false);

    // Check for protection message
    let mut messages = Vec::new();
    while let Some(msg) = sys.pop_message() {
      messages.push(msg);
    }
    assert!(messages
      .iter()
      .any(|m| m.contains("-Protect- Turning filter OFF")));
  }

  #[test]
  fn valve_change_stops_both_filter_types() {
    let mut sys = System::<HasOSMech, HasOSLights> {
      filter: Filter {
        running_schedule: true,
        quick_clean: true,
      },
      main_valve_orientation: PoolOrSpa::Pool,
      ..Default::default()
    };

    sys.set_main_valves(PoolOrSpa::Spa);

    // Both filter types should be stopped
    assert_eq!(sys.filter.running_schedule, false);
    assert_eq!(sys.filter.quick_clean, false);
  }

  // Message queue tests
  #[test]
  fn jets_toggle_logs_correct_messages() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();

    sys.toggle_jets();
    let mut messages = Vec::new();
    while let Some(msg) = sys.pop_message() {
      messages.push(msg);
    }
    assert!(messages.iter().any(|m| m.contains("Jets ON")));

    sys.toggle_jets();
    let mut messages = Vec::new();
    while let Some(msg) = sys.pop_message() {
      messages.push(msg);
    }
    assert!(messages.iter().any(|m| m.contains("Jets OFF")));
  }

  #[test]
  fn heater_toggle_logs_correct_messages() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();

    sys.set_heater_on(true);
    let mut messages = Vec::new();
    while let Some(msg) = sys.pop_message() {
      messages.push(msg);
    }
    assert!(messages.iter().any(|m| m.contains("Heater ON")));

    sys.set_heater_on(false);
    let mut messages = Vec::new();
    while let Some(msg) = sys.pop_message() {
      messages.push(msg);
    }
    assert!(messages.iter().any(|m| m.contains("Heater OFF")));
  }

  #[test]
  fn auto_spa_logs_start_and_complete_messages() {
    let mut sys = System::<HasOSMech, HasOSLights>::default();

    sys.auto_spa(Some(true));

    let mut messages = Vec::new();
    while let Some(msg) = sys.pop_message() {
      messages.push(msg);
    }

    assert!(messages
      .iter()
      .any(|m| m.contains("Starting Spa... Enjoy! xo")));
    assert!(messages.iter().any(|m| m.contains("Complete: Spa Mode")));
  }
}
