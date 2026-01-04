use crate::message_queue::MessageQueue;
use core::fmt;

#[cfg(not(target_os = "none"))]
use std::thread;
#[cfg(not(target_os = "none"))]
use std::time::Duration;

impl fmt::Display for PoolOrSpa {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      PoolOrSpa::Spa => write!(f, "SPA"),
      PoolOrSpa::Pool => write!(f, "POOL"),
    }
  }
}
impl fmt::Display for PoolValve {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      PoolValve::Blend => write!(f, "Blend"),
      PoolValve::Skimmer => write!(f, "Skimmer"),
      PoolValve::Vacuum => write!(f, "Vacuum"),
    }
  }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PoolOrSpa {
  Pool,
  Spa,
}
#[derive(Copy, Clone, PartialEq)]
pub enum PoolValve {
  Vacuum,
  Skimmer,
  Blend,
}

pub struct Filter {
  pub running_schedule: bool,
  pub quick_clean: bool,
}
pub struct Heater {
  pub mode: PoolOrSpa,
  pub on: bool,
}

pub struct PrevState {
  pub filter: Filter,
  pub heater: Heater,
  pub main_valve_orientation: Option<PoolOrSpa>,
}

pub trait Mech {
  fn delay_secs(&self, secs: u64);

  // Filter
  fn set_quick_clean(&self, v: bool) -> bool;
  fn mech_set_filter_sched(&self, v: bool) -> bool;

  // Valves
  fn mech_pool_valve_to(&self, p: PoolValve) -> bool;
  fn mech_main_valve_to(&self, m: PoolOrSpa) -> bool;

  // Heater
  fn heater_on_toggle(&self, b: bool) -> bool;
  fn heater_mode_toggle(&self, m: PoolOrSpa) -> bool;

  // Jets
  fn jets_on_toggle(&self, b: bool) -> bool;
}

pub trait Lights {
  fn in_progress(&self, b: bool) -> bool;
  fn filter_schedule(&self, b: bool) -> bool;
  fn heater_on(&self, b: bool) -> bool;
  fn jets_on(&self, b: bool) -> bool;
  fn auto_spa(&self, b: bool) -> bool;
  fn heater_mode(&self, m: PoolOrSpa) -> bool;
  fn quick_clean(&self, b: bool) -> bool;
  fn main_valve_orientation(&self, b: PoolOrSpa) -> bool;
}

pub struct HasOSMech;
pub struct HasOSLights;

impl Mech for HasOSMech {
  fn delay_secs(&self, secs: u64) {
    thread::sleep(Duration::from_secs(secs));
  }

  fn mech_main_valve_to(&self, _: PoolOrSpa) -> bool {
    true
  }

  fn set_quick_clean(&self, _: bool) -> bool {
    true
  }
  fn mech_set_filter_sched(&self, _: bool) -> bool {
    true
  }
  fn mech_pool_valve_to(&self, _: PoolValve) -> bool {
    true
  }

  // Heater:
  fn heater_on_toggle(&self, _: bool) -> bool {
    true
  }
  fn heater_mode_toggle(&self, _: PoolOrSpa) -> bool {
    true
  }

  // JETS
  fn jets_on_toggle(&self, _: bool) -> bool {
    true
  }
}
impl Lights for HasOSLights {
  fn in_progress(&self, _: bool) -> bool {
    true
  }
  fn filter_schedule(&self, _: bool) -> bool {
    true
  }
  fn heater_on(&self, _: bool) -> bool {
    true
  }
  fn jets_on(&self, _: bool) -> bool {
    true
  }
  fn auto_spa(&self, _: bool) -> bool {
    true
  }
  fn heater_mode(&self, _: PoolOrSpa) -> bool {
    true
  }

  fn quick_clean(&self, _: bool) -> bool {
    true
  }
  fn main_valve_orientation(&self, _: PoolOrSpa) -> bool {
    true
  }
}

impl Default for System<HasOSMech, HasOSLights> {
  fn default() -> Self {
    Self {
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
      mech: HasOSMech,
      lights: HasOSLights,
      internal_test: true,
      prev_state: None,
      message_queue: MessageQueue::new(),
      auto_spa_mode: false,
    }
  }
}

pub struct System<M: Mech, L: Lights> {
  pub main_valve_orientation: PoolOrSpa,
  pub pool_valve_orientation: PoolValve,
  pub filter: Filter,
  pub heater: Heater,
  pub jets_on: bool,
  pub errors: [Option<u32>; 10],
  pub in_progress: bool,
  pub mech: M,
  pub lights: L,
  pub internal_test: bool,
  pub prev_state: Option<PrevState>,
  pub message_queue: MessageQueue<48>,
  pub auto_spa_mode: bool,
}
