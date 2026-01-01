use core::fmt;
#[cfg(not(target_os = "none"))]
use std::thread;
#[cfg(not(target_os = "none"))]
use std::time::Duration;

impl fmt::Display for SystemMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemMode::Spa => write!(f, "Spa"),
            SystemMode::Pool => write!(f, "Pool"),
        }
    }
}
impl fmt::Display for PoolMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoolMode::Blend => write!(f, "Blend"),
            PoolMode::Skimmer => write!(f, "Skimmer"),
            PoolMode::Vacuum => write!(f, "Vacuum"),
        }
    }
}
pub struct System<P: Platform> {
    pub system_mode: SystemMode,
    pub pool_mode: PoolMode,
    pub filter_on: bool,
    pub heater_on: bool,
    pub jets_on: bool,
    pub valves_changing: bool,
    pub errors: [Option<u32>; 10],
    pub in_progress: bool,
    pub platform: P,
}

#[derive(Copy, Clone)]
pub enum SystemMode {
    Spa,
    Pool,
}
#[derive(Copy, Clone)]
pub enum PoolMode {
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
    fn mech_pool_valve_to(&self, p: PoolMode);
    fn mech_main_valve_to(&self, m: SystemMode);
    fn mech_heater(&self, v: bool);
    fn mech_jets(&self, v: bool);
}

impl Platform for HasOS {
    fn delay_secs(&self, secs: u64) {
        thread::sleep(Duration::from_secs(secs));
    }

    fn mech_main_valve_to(&self, _: SystemMode) {}
    fn mech_filter_on(&self) {}
    fn mech_filter_off(&self) {}
    fn mech_pool_valve_to(&self, _: PoolMode) {}
    fn mech_heater(&self, _: bool) {}
    fn mech_jets(&self, _: bool) {}
}

impl<P: Platform> System<P> {
    pub fn new(platform: P) -> Self {
        System {
            system_mode: SystemMode::Pool,
            pool_mode: PoolMode::Skimmer,
            filter_on: false,
            heater_on: false,
            jets_on: false,
            valves_changing: false,
            errors: [None; 10],
            in_progress: false,
            platform,
        }
    }

    // JETS
    pub fn start_jets(&mut self) {
        println!("Turning jets on.");
        self.platform.mech_jets(true);
        self.jets_on = true;
    }
    pub fn stop_jets(&mut self) {
        println!("Turning jets off.");
        self.platform.mech_jets(false);
        self.jets_on = false;
    }

    // FILTER
    pub fn start_filter(&mut self) {
        println!("Start: Turning filter on.");
        self.filter_on = true;
        self.in_progress = true;
        self.platform.mech_filter_on();
        self.platform.delay_secs(10);
        self.in_progress = false;
        println!("Complete: Filter primed and running.");
    }
    pub fn stop_filter(&mut self) {
        println!("Start: Turning filter off.");
        self.filter_on = false;
        self.in_progress = true;
        self.platform.mech_filter_off();
        self.platform.delay_secs(5);
        self.in_progress = false;
        println!("Complete: Filter Off.");
    }

    // HEATER
    pub fn start_heater(&mut self) {
        println!("Turning heater on");
        self.platform.mech_heater(true);
        self.heater_on = true;
    }
    pub fn stop_heater(&mut self) {
        println!("Turning heater off");
        self.platform.mech_heater(true);
        self.heater_on = false;
    }

    // VALVES
    pub fn change_main_valves(&mut self, m: SystemMode) {
        println!("Start: Changing main valve orientation to {}.", m);
        let filter_was_running = self.filter_on;

        if filter_was_running {
            self.stop_filter();
        }

        self.in_progress = true;
        self.platform.mech_filter_off();
        self.platform.mech_main_valve_to(m);
        self.platform.delay_secs(10);
        self.in_progress = false;

        if filter_was_running {
            self.start_filter();
        }

        println!("Complete: Valves Changed to {} mode", m);
    }

    pub fn change_pool_valve(&mut self, m: PoolMode) {
        println!("Start: Changing pool valve to {} mode", m);
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
        println!("Complete: Valves Changed to {} mode", m);
    }
}
