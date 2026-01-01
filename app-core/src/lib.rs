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
}

#[derive(Copy, Clone, PartialEq)]
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

pub struct HasOS;
pub struct NoOS;

//platform specific behavior:
pub trait Platform {
    fn delay_secs(&self, secs: u64);
    fn mech_filter_on(&self);
    fn mech_filter_off(&self);
    fn mech_pool_valve_to(&self, p: PoolValve);
    fn mech_main_valve_to(&self, m: PoolOrSpa);
    fn mech_heater_toggle(&self, v: bool);
    fn mech_heat_mode_toggle(&self, m: PoolOrSpa);
    fn mech_jets_toggle(&self, v: bool);
    fn mech_quick_clean_toggle(&self, v: bool);
}

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
        }
    }

    // JETS
    pub fn start_jets(&mut self) {
        if self.jets_on {
            return;
        }
        println!("Turning jets on.");
        self.platform.mech_jets_toggle(true);
        self.jets_on = true;
    }
    pub fn stop_jets(&mut self) {
        if !self.jets_on {
            return;
        }
        println!("Turning jets off.");
        self.platform.mech_jets_toggle(false);
        self.jets_on = false;
    }

    // FILTER
    pub fn start_filter(&mut self) {
        if self.filter_on {
            return;
        }
        println!("Start: Turning filter on.");
        self.filter_on = true;
        self.in_progress = true;
        self.platform.mech_filter_on();
        self.platform.delay_secs(10);
        self.in_progress = false;
        println!("Complete: Filter primed and running.");
    }
    pub fn stop_filter(&mut self) {
        if !self.filter_on {
            return;
        }
        println!("Start: Turning filter off.");
        self.filter_on = false;
        self.in_progress = true;
        self.platform.mech_filter_off();
        self.platform.delay_secs(5);
        self.in_progress = false;
        println!("Complete: Filter Off.");
    }
    pub fn start_quick_clean(&mut self) {
        if self.quick_clean_on {
            return;
        }
        println!("Start: Turning quick clean on.");
        self.quick_clean_on = true;
        self.in_progress = true;
        self.platform.mech_quick_clean_toggle(true);

        if !self.filter_on {
            println!("Filter was off.  Need time to start...");
            self.platform.delay_secs(10);
        }

        self.in_progress = false;
        println!("Complete: Quick Clean is On")
    }

    // HEATER
    pub fn start_heater(&mut self) {
        if self.heater_on {
            return;
        }
        println!("Turning heater on");
        self.platform.mech_heater_toggle(true);
        self.heater_on = true;
    }
    pub fn stop_heater(&mut self) {
        if !self.heater_on {
            return;
        }
        println!("Turning heater off");
        self.platform.mech_heater_toggle(true);
        self.heater_on = false;
    }
    pub fn toggle_heat_mode(&mut self, m: PoolOrSpa) {
        if self.heat_mode == m {
            return;
        }
        println!("Toggling heat mode to {}.", m);
        self.heat_mode = m;
        self.platform.mech_heat_mode_toggle(m);
    }

    // VALVES
    pub fn change_main_valves(&mut self, m: PoolOrSpa) {
        if self.main_valve_orientation == m {
            return;
        }

        println!("Start: Changing main valve orientation to {}.", m);
        let filter_was_running = self.filter_on;

        if filter_was_running {
            self.stop_filter();
        }

        self.in_progress = true;
        self.platform.mech_main_valve_to(m);
        self.platform.delay_secs(10);
        self.in_progress = false;

        if filter_was_running {
            self.start_filter();
        }

        println!("Complete: Valves Changed to {} mode.", m);
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

        self.in_progress = true;
        self.platform.mech_filter_off();
        self.platform.mech_pool_valve_to(m);
        self.platform.delay_secs(10);
        self.in_progress = false;

        if filter_was_running {
            self.start_filter();
        }
        println!("Complete: Valves Changed to {} mode.", m);
    }

    // MODES
    pub fn enter_spa_mode(&mut self) {
        if self.heater_on && self.quick_clean_on && self.heat_mode == PoolOrSpa::Spa {
            return;
        }

        println!("Start: Changing to Spa Mode");

        if self.main_valve_orientation == PoolOrSpa::Pool {
            self.change_main_valves(PoolOrSpa::Spa);
        }

        self.toggle_heat_mode(PoolOrSpa::Spa);
        self.start_heater();
        self.start_quick_clean();

        println!("Complete: Spa Mode");
    }
    pub fn enter_pool_mode(&mut self) {
        if self.main_valve_orientation == PoolOrSpa::Spa {
            self.change_main_valves(PoolOrSpa::Pool);
        } else {
            println!("Main valves are already in Pool Mode.");
        }
    }
}
