use core::fmt;
#[cfg(not(target_os = "none"))]
use std::thread;
#[cfg(not(target_os = "none"))]
use std::time::Duration;

impl fmt::Display for PoolOrSpa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoolOrSpa::Spa => write!(f, "Spa"),
            PoolOrSpa::Pool => write!(f, "Pool"),
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

//platform specific behavior:
pub trait Platform {
    fn delay_secs(&self, secs: u64);

    // Filter
    fn mech_filter_on(&self) -> bool;
    fn mech_filter_off(&self) -> bool;
    fn set_quick_clean(&self, v: bool) -> bool;
    fn mech_set_filter_sched(&self, v: bool) -> bool;

    // Valves
    fn mech_pool_valve_to(&self, p: PoolValve) -> bool;
    fn mech_main_valve_to(&self, m: PoolOrSpa) -> bool;
    fn main_valve_light_toggle(&self, m: PoolOrSpa) -> bool;

    // Heater
    fn heater_on_toggle(&self, b: bool) -> bool;
    fn heater_mode_toggle(&self, m: PoolOrSpa) -> bool;
    fn heater_mode_light_toggle(&self, m: PoolOrSpa) -> bool;
    fn heater_on_light_toggle(&self, _: bool) -> bool;

    // Jets
    fn jets_on_toggle(&self, b: bool) -> bool;
    fn jets_light_toggle(&self, _: bool) -> bool;

    // Lights
    fn mech_toggle_quick_clean_light(&self, b: bool) -> bool;
    fn mech_toggle_progress_light(&self, b: bool) -> bool;
    fn mech_toggle_sched_light(&self, _: bool) -> bool;
    fn mech_toggle_heater_mode_light(&self, m: PoolOrSpa) -> bool;
}
pub struct HasOS;
impl Platform for HasOS {
    fn delay_secs(&self, secs: u64) {
        thread::sleep(Duration::from_secs(secs));
    }

    fn mech_main_valve_to(&self, _: PoolOrSpa) -> bool {
        true
    }
    fn mech_filter_on(&self) -> bool {
        true
    }
    fn set_quick_clean(&self, _: bool) -> bool {
        true
    }
    fn mech_set_filter_sched(&self, _: bool) -> bool {
        true
    }
    fn mech_filter_off(&self) -> bool {
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
    fn heater_on_light_toggle(&self, _: bool) -> bool {
        true
    }
    fn heater_mode_light_toggle(&self, _: PoolOrSpa) -> bool {
        true
    }

    // JETS
    fn jets_on_toggle(&self, _: bool) -> bool {
        true
    }
    fn jets_light_toggle(&self, _: bool) -> bool {
        true
    }

    // Lights
    fn mech_toggle_progress_light(&self, _: bool) -> bool {
        true
    }
    fn mech_toggle_sched_light(&self, _: bool) -> bool {
        true
    }
    fn mech_toggle_quick_clean_light(&self, _: bool) -> bool {
        true
    }
    fn mech_toggle_heater_mode_light(&self, _: PoolOrSpa) -> bool {
        true
    }
    fn main_valve_light_toggle(&self, _: PoolOrSpa) -> bool {
        true
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

// Main system state:
pub struct System<P: Platform> {
    pub main_valve_orientation: PoolOrSpa,
    pub pool_valve_orientation: PoolValve,
    pub filter_on: bool,
    pub filter: Filter,
    pub heater: Heater,
    pub jets_on: bool,
    pub errors: [Option<u32>; 10],
    pub in_progress: bool,
    pub platform: P,
    pub internal_test: bool,
    pub prev_state: Option<PrevState>,
}

// For testing purposes on desktop machine:
impl Default for System<HasOS> {
    fn default() -> Self {
        Self {
            main_valve_orientation: PoolOrSpa::Pool,
            pool_valve_orientation: PoolValve::Skimmer,
            filter_on: false,
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
            platform: HasOS,
            internal_test: true,
            prev_state: None,
        }
    }
}

// All the state methods:
impl<P: Platform> System<P> {
    pub fn new(platform: P) -> Self {
        System {
            main_valve_orientation: PoolOrSpa::Pool,
            pool_valve_orientation: PoolValve::Skimmer,
            filter_on: false,
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
            platform,
            internal_test: false,
            prev_state: None,
        }
    }

    // JETS ✅
    pub fn toggle_jets(&mut self) -> bool {
        if self.jets_on {
            println!("Turning Jets Off");
        } else {
            println!("Turning Jets On");
        }

        let new = !self.jets_on;

        self.platform.jets_on_toggle(new);
        self.platform.jets_light_toggle(new);

        self.jets_on = new;

        true
    }

    // FILTER
    pub fn filter_delay(&mut self) -> bool {
        println!("Start: Turning filter on.");

        self.with_progress_light(|s| {
            if !s.internal_test {
                s.platform.delay_secs(10);
            }
        });

        println!("Complete: Filter primed and running.");
        true
    }
    pub fn toggle_quick_clean(&mut self) -> bool {
        let new = !self.filter.quick_clean;

        self.platform.mech_toggle_quick_clean_light(new);
        self.platform.set_quick_clean(new);

        if self.filter.quick_clean {
            println!("Turning off quick clean");
            self.filter.quick_clean = false;
        } else {
            println!("Turning on Quick Clean");
            self.start_quick_clean();
        }

        true
    }
    pub fn toggle_filter_schedule(&mut self) -> bool {
        let new = !self.filter.running_schedule;
        self.platform.mech_toggle_sched_light(new);
        self.filter.running_schedule = new;

        if !self.filter.running_schedule {
            self.filter_delay();
        }

        true
    }
    pub fn stop_filter(&mut self) -> bool {
        if !self.filter.quick_clean && !self.filter.running_schedule {
            return false;
        }
        println!("Start: Turning filter off");

        self.with_progress_light(|s| {
            s.platform.set_quick_clean(false);
            s.platform.mech_set_filter_sched(false);

            if !s.internal_test {
                s.platform.delay_secs(5);
            }

            s.filter.quick_clean = false;
            s.filter.running_schedule = false;
        });

        println!("End: Filter is off");
        true
    }
    pub fn start_quick_clean(&mut self) -> bool {
        if self.filter.quick_clean {
            return false;
        }

        println!("Start: Turning quick clean on");

        self.platform.mech_toggle_quick_clean_light(true);
        self.platform.set_quick_clean(true);
        self.filter_delay();
        self.filter.quick_clean = true;

        println!("Complete: Quick clean is on");

        true
    }

    // HEATER ✅
    pub fn set_heater_on(&mut self, b: bool) -> bool {
        if self.heater.on == b {
            return false;
        }

        self.platform.heater_on_toggle(b);
        self.platform.heater_on_light_toggle(b);
        self.heater.on = b;

        if b {
            println!("Turning heater on");
        } else {
            println!("Turning heater off");
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
    pub fn toggle_heat_mode(&mut self, m: PoolOrSpa) -> bool {
        if self.heater.mode == m {
            return false;
        }

        println!("Toggling heat mode to {}", m);
        self.heater.mode = m;
        self.platform.heater_mode_light_toggle(m);
        self.platform.heater_mode_toggle(m);
        return true;
    }

    // VALVES
    pub fn set_main_valves(&mut self, m: PoolOrSpa) -> bool {
        if self.main_valve_orientation == m {
            return false;
        }

        self.platform.main_valve_light_toggle(m);
        println!("Start: Changing main valve orientation to {}", m);

        if self.filter.quick_clean || self.filter.running_schedule {
            println!("Turning off filter first");
            self.stop_filter();
        }

        let prev_state = Some(PrevState {
            heater: Heater {
                mode: self.heater.mode,
                on: self.heater.on,
            },
            filter: Filter {
                running_schedule: self.filter.running_schedule,
                quick_clean: self.filter.running_schedule,
            },
            main_valve_orientation: None,
        });

        self.with_progress_light(|s| {
            s.platform.mech_main_valve_to(m);
            if !s.internal_test {
                s.platform.delay_secs(10);
            }
            s.main_valve_orientation = m;
        });

        println!("Complete: Valves Changed to {} mode.", m);
        self.restore_previous_state(prev_state);

        true
    }
    pub fn toggle_main_valves(&mut self) {
        match self.main_valve_orientation {
            PoolOrSpa::Pool => self.set_main_valves(PoolOrSpa::Spa),
            PoolOrSpa::Spa => self.set_main_valves(PoolOrSpa::Pool),
        };
    }
    pub fn set_pool_valve(&mut self, m: PoolValve) {
        if self.pool_valve_orientation == m {
            return;
        }

        println!("Start: Changing pool valve to {} mode.", m);
        let filter_was_running = self.filter_on;

        if filter_was_running {
            self.stop_filter();
        }

        self.with_progress_light(|s| {
            s.platform.mech_filter_off();
            s.platform.mech_pool_valve_to(m);

            if !s.internal_test {
                s.platform.delay_secs(10);
            }
        });

        if filter_was_running {
            self.filter_delay();
        }
        println!("Complete: Valves Changed to {} mode.", m);
    }

    // ROUTINES
    pub fn auto_spa(&mut self) -> (bool, bool, bool) {
        if self.heater.on && self.filter.quick_clean && self.heater.mode == PoolOrSpa::Spa {
            return (false, false, false);
        }

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

        println!("Starting Spa... Enjoy! xo");

        if self.main_valve_orientation == PoolOrSpa::Pool {
            self.set_main_valves(PoolOrSpa::Spa);
        }

        let op1 = self.toggle_heat_mode(PoolOrSpa::Spa);
        let op2 = self.set_heater_on(true);
        let op3 = self.start_quick_clean();

        if !self.internal_test {
            self.platform.delay_secs(15);
        } else {
            println!("Delay of 3 hours");
        }
        self.restore_previous_state(prev_state);

        println!("Complete: Spa Mode");
        return (op1, op2, op3);
    }
    fn restore_previous_state(&mut self, s: Option<PrevState>) -> bool {
        if let Some(_) = s {
            println!("Restoring equipment to it's previous state");
            return true;
        } else {
            false
        }
    }

    // LIGHTS
    fn with_progress_light<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.in_progress = true;
        self.platform.mech_toggle_progress_light(true);
        f(self);
        self.platform.mech_toggle_progress_light(false);
        self.in_progress = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_does_not_start_if_running() {
        let mut sys = System::<HasOS> {
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
        let mut sys = System::<HasOS> {
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
        let mut sys = System::<HasOS> {
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
        let mut sys = System::<HasOS>::default();
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
        let mut sys = System::<HasOS>::default();
        assert_eq!(sys.auto_spa(), (true, true, true));
        assert_eq!(sys.filter.quick_clean, true);
        assert_eq!(sys.heater.on, true);
        assert_eq!(sys.main_valve_orientation, PoolOrSpa::Spa);
        assert_eq!(sys.auto_spa(), (false, false, false));

        // heater is already running for pool
        let mut sys = System::<HasOS> {
            heater: Heater {
                mode: PoolOrSpa::Pool,
                on: true,
            },
            ..Default::default()
        };
        let result = sys.auto_spa();
        assert_eq!(result, (true, false, true));
        println!("-------------------------");
        // heater is already running for spa
        let mut sys = System::<HasOS> {
            heater: Heater {
                mode: PoolOrSpa::Spa,
                on: true,
            },
            ..Default::default()
        };
        let result = sys.auto_spa();
        assert_eq!(result, (false, false, true));

        // already in spa mode:
        let mut sys = System::<HasOS> {
            heater: Heater {
                mode: PoolOrSpa::Spa,
                on: true,
            },
            filter: Filter {
                running_schedule: false,
                quick_clean: true,
            },
            ..Default::default()
        };
        let result = sys.auto_spa();
        assert_eq!(result, (false, false, false))
    }

    #[test]
    fn set_main_valves_is_safe() {
        let mut sys = System::<HasOS> {
            filter_on: true,
            main_valve_orientation: PoolOrSpa::Spa,
            ..Default::default()
        };
        let result = sys.set_main_valves(PoolOrSpa::Pool);
        assert_eq!(result, true);
    }

    #[test]
    fn set_main_valves_does_not_run_if_uneeded() {
        let mut sys = System::<HasOS> {
            main_valve_orientation: PoolOrSpa::Pool,
            ..Default::default()
        };
        let result = sys.set_main_valves(PoolOrSpa::Pool);
        assert_eq!(result, false);
        assert_eq!(sys.main_valve_orientation, PoolOrSpa::Pool);
    }
    #[test]
    fn set_main_valves_runs_if_uneeded() {
        let mut sys = System::<HasOS> {
            main_valve_orientation: PoolOrSpa::Pool,
            ..Default::default()
        };
        let result = sys.set_main_valves(PoolOrSpa::Spa);
        assert_eq!(result, true);
        assert_eq!(sys.main_valve_orientation, PoolOrSpa::Spa);
    }

    #[test]
    fn jets_behave_expected() {
        let mut sys = System::<HasOS>::default();
        assert_eq!(sys.jets_on, false);
        sys.toggle_jets();
        assert_eq!(sys.jets_on, true);
        sys.toggle_jets();
        assert_eq!(sys.jets_on, false);
    }
}
