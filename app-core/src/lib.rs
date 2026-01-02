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
    fn mech_filter_on(&self);
    fn mech_filter_off(&self);
    fn mech_pool_valve_to(&self, p: PoolValve);
    fn mech_main_valve_to(&self, m: PoolOrSpa);
    fn mech_heater_toggle(&self, b: bool);
    fn mech_heat_mode_toggle(&self, m: PoolOrSpa);
    fn mech_jets_toggle(&self, b: bool);
    fn mech_quick_clean_toggle(&self, b: bool);
    fn mech_progress_light_toggle(&self, b: bool);
}
pub struct HasOS;
impl Platform for HasOS {
    fn delay_secs(&self, secs: u64) {
        thread::sleep(Duration::from_secs(secs));
    }
    fn mech_main_valve_to(&self, _: PoolOrSpa) {}
    fn mech_filter_on(&self) {}
    fn mech_filter_off(&self) {}
    fn mech_pool_valve_to(&self, _: PoolValve) {}
    fn mech_heater_toggle(&self, _: bool) {}
    fn mech_heat_mode_toggle(&self, _: PoolOrSpa) {}
    fn mech_jets_toggle(&self, _: bool) {}
    fn mech_quick_clean_toggle(&self, _: bool) {}

    // LIGHTS
    fn mech_progress_light_toggle(&self, _: bool) {}
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

// Main system state
pub struct System<P: Platform> {
    pub main_valve_orientation: PoolOrSpa,
    pub pool_valve_orientation: PoolValve,
    pub heat_mode: PoolOrSpa,
    pub filter_on: bool,
    pub heater_on: bool,
    pub jets_on: bool,
    pub errors: [Option<u32>; 10],
    pub in_progress: bool,
    pub quick_clean_on: bool,
    pub platform: P,
    pub internal_test: bool,
}

// For testing purposes on desktop machine:
impl Default for System<HasOS> {
    fn default() -> Self {
        Self {
            main_valve_orientation: PoolOrSpa::Pool,
            pool_valve_orientation: PoolValve::Skimmer,
            heat_mode: PoolOrSpa::Pool,
            filter_on: false,
            heater_on: false,
            jets_on: false,
            errors: [None; 10],
            in_progress: false,
            quick_clean_on: false,
            platform: HasOS,
            internal_test: true,
        }
    }
}

impl<P: Platform> System<P> {
    pub fn new(platform: P) -> Self {
        System {
            main_valve_orientation: PoolOrSpa::Pool,
            pool_valve_orientation: PoolValve::Skimmer,
            heat_mode: PoolOrSpa::Pool,
            filter_on: false,
            heater_on: false,
            jets_on: false,
            errors: [None; 10],
            in_progress: false,
            platform,
            quick_clean_on: false,
            internal_test: false,
        }
    }

    // JETS
    pub fn toggle_jets(&mut self) -> bool {
        if self.jets_on {
            println!("Turning Jets Off");
        } else {
            println!("Turning Jets On");
        }

        self.platform.mech_jets_toggle(!self.jets_on);
        self.jets_on = !self.jets_on;

        return true;
    }

    // FILTER
    pub fn start_filter(&mut self) -> bool {
        if self.filter_on {
            return false;
        }
        println!("Start: Turning filter on.");
        self.filter_on = true;

        self.with_progress_light(|s| {
            s.platform.mech_filter_on();
            if !s.internal_test {
                s.platform.delay_secs(10);
            }
        });

        println!("Complete: Filter primed and running.");
        return true;
    }
    pub fn stop_filter(&mut self) -> bool {
        if !self.filter_on {
            return false;
        }
        println!("Start: Turning filter off.");
        self.filter_on = false;

        self.with_progress_light(|s| {
            s.platform.mech_filter_off();
            if !s.internal_test {
                s.platform.delay_secs(5);
            }
        });

        println!("Complete: Filter Off.");
        return true;
    }
    pub fn start_quick_clean(&mut self) -> bool {
        if self.quick_clean_on {
            return false;
        }

        println!("Start: Turning quick clean on.");
        self.quick_clean_on = true;

        self.with_progress_light(|s| {
            s.platform.mech_quick_clean_toggle(true);

            if !s.filter_on {
                println!("Filter was off. Need time to start...");
                if !s.internal_test {
                    s.platform.delay_secs(10);
                }
            }
        });

        println!("Complete: Quick Clean is On");
        true
    }

    // HEATER
    pub fn start_heater(&mut self) -> bool {
        if self.heater_on {
            return false;
        }
        println!("Turning heater on");
        self.platform.mech_heater_toggle(true);
        self.heater_on = true;
        return true;
    }
    pub fn stop_heater(&mut self) -> bool {
        if !self.heater_on {
            return false;
        }
        println!("Turning heater off");
        self.platform.mech_heater_toggle(false);
        self.heater_on = false;
        return true;
    }
    pub fn toggle_heat_mode(&mut self, m: PoolOrSpa) -> bool {
        if self.heat_mode == m {
            return false;
        }
        println!("Toggling heat mode to {}.", m);
        self.heat_mode = m;
        self.platform.mech_heat_mode_toggle(m);
        return true;
    }

    // VALVES
    pub fn change_main_valves(&mut self, m: PoolOrSpa) -> (bool, bool) {
        if self.main_valve_orientation == m {
            return (false, false);
        }

        println!("Start: Changing main valve orientation to {}.", m);

        let filter_was_running = self.filter_on;
        let mut filter_was_stopped = false;
        let mut filter_was_restarted = false;

        if filter_was_running {
            self.stop_filter();
            filter_was_stopped = true;
        }

        self.with_progress_light(|s| {
            s.platform.mech_main_valve_to(m);
            if !s.internal_test {
                s.platform.delay_secs(10);
            }
        });

        if filter_was_stopped {
            self.start_filter();
            filter_was_restarted = true;
        }

        self.main_valve_orientation = m;

        println!("Complete: Valves Changed to {} mode.", m);
        (filter_was_stopped, filter_was_restarted)
    }

    pub fn change_pool_valve(&mut self, m: PoolValve) {
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
            self.start_filter();
        }
        println!("Complete: Valves Changed to {} mode.", m);
    }

    // MODES
    pub fn auto_spa(&mut self) -> (bool, bool, bool) {
        if self.heater_on && self.quick_clean_on && self.heat_mode == PoolOrSpa::Spa {
            return (false, false, false);
        }

        println!("Start: Changing to Spa Mode");

        if self.main_valve_orientation == PoolOrSpa::Pool {
            self.change_main_valves(PoolOrSpa::Spa);
        }

        let op1 = self.toggle_heat_mode(PoolOrSpa::Spa);
        let op2 = self.start_heater();
        let op3 = self.start_quick_clean();

        println!("Complete: Spa Mode");
        return (op1, op2, op3);
    }
    pub fn enter_pool_mode(&mut self) -> bool {
        if self.main_valve_orientation == PoolOrSpa::Spa {
            self.change_main_valves(PoolOrSpa::Pool);
            return true;
        } else {
            println!("Main valves are already in Pool Mode.");
            return false;
        }
    }

    // LIGHTS
    fn with_progress_light<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.in_progress = true;
        self.platform.mech_progress_light_toggle(true);
        f(self);
        self.platform.mech_progress_light_toggle(false);
        self.in_progress = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_does_not_start_if_running() {
        let mut sys = System::<HasOS> {
            filter_on: true,
            ..Default::default()
        };
        assert_eq!(sys.start_filter(), false);
        assert_eq!(sys.filter_on, true);
        assert_eq!(sys.start_filter(), false);
        assert_eq!(sys.filter_on, true);
    }

    #[test]
    fn filter_does_not_stop_if_stopped() {
        let mut sys = System::<HasOS> {
            filter_on: false,
            ..Default::default()
        };
        assert_eq!(sys.stop_filter(), false);
        assert_eq!(sys.filter_on, false);
        assert_eq!(sys.stop_filter(), false);
        assert_eq!(sys.filter_on, false);
    }

    #[test]
    fn filter_status_changes_correctly() {
        let mut sys = System::<HasOS>::default();
        sys.start_filter();
        assert_eq!(sys.filter_on, true);
        assert_eq!(sys.start_filter(), false);
        assert_eq!(sys.filter_on, true);
        sys.stop_filter();
        assert_eq!(sys.filter_on, false);
        assert_eq!(sys.stop_filter(), false);
        assert_eq!(sys.filter_on, false);
    }

    #[test]
    fn heater_status_changes_correctly() {
        let mut sys = System::<HasOS>::default();
        sys.start_heater();
        assert_eq!(sys.heater_on, true);
        assert_eq!(sys.start_heater(), false);
        assert_eq!(sys.heater_on, true);
        sys.stop_heater();
        assert_eq!(sys.heater_on, false);
        assert_eq!(sys.stop_heater(), false);
        assert_eq!(sys.heater_on, false);
    }

    #[test]
    fn spa_mode_behaves_expected() {
        let mut sys = System::<HasOS>::default();
        assert_eq!(sys.auto_spa(), (true, true, true));
        assert_eq!(sys.quick_clean_on, true);
        assert_eq!(sys.heater_on, true);
        assert_eq!(sys.main_valve_orientation, PoolOrSpa::Spa);
        assert_eq!(sys.auto_spa(), (false, false, false));

        // heater is already running for pool
        let mut sys = System::<HasOS> {
            heater_on: true,
            ..Default::default()
        };
        let result = sys.auto_spa();
        assert_eq!(result, (true, false, true));

        // heater running for spa
        let mut sys = System::<HasOS> {
            heater_on: true,
            heat_mode: PoolOrSpa::Spa,
            ..Default::default()
        };
        let result = sys.auto_spa();
        assert_eq!(result, (false, false, true));

        // already in spa mode:
        let mut sys = System::<HasOS> {
            heater_on: true,
            heat_mode: PoolOrSpa::Spa,
            quick_clean_on: true,
            ..Default::default()
        };
        let result = sys.auto_spa();
        assert_eq!(result, (false, false, false))
    }

    #[test]
    fn change_main_valves_is_safe() {
        let mut sys = System::<HasOS> {
            filter_on: true,
            main_valve_orientation: PoolOrSpa::Spa,
            ..Default::default()
        };
        let result = sys.change_main_valves(PoolOrSpa::Pool);
        assert_eq!(result, (true, true));
    }

    #[test]
    fn change_main_valves_runs_only_when_needed() {
        let mut sys = System::<HasOS> {
            main_valve_orientation: PoolOrSpa::Spa,
            ..Default::default()
        };
        let result = sys.change_main_valves(PoolOrSpa::Spa);
        assert_eq!(result, (false, false));
        let mut sys = System::<HasOS> {
            main_valve_orientation: PoolOrSpa::Pool,
            ..Default::default()
        };
        let result = sys.change_main_valves(PoolOrSpa::Pool);
        assert_eq!(result, (false, false));
    }

    #[test]
    fn enter_pool_mode_runs_only_when_needed() {
        let mut sys = System::<HasOS> {
            main_valve_orientation: PoolOrSpa::Pool,
            ..Default::default()
        };
        assert_eq!(sys.enter_pool_mode(), false);
        let mut sys = System::<HasOS> {
            main_valve_orientation: PoolOrSpa::Spa,
            ..Default::default()
        };
        assert_eq!(sys.enter_pool_mode(), true)
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
