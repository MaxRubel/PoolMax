pub struct System {
    pub system_mode: SystemMode,
    pub pool_mode: PoolMode,
    pub filter_on: bool,
    pub heater_on: bool,
    pub jets_on: bool,
    pub valves_changing: bool,
    pub errors: [Option<u32>; 10],
}

pub enum SystemMode {
    Spa,
    Pool,
}

pub enum PoolMode {
    Vacuum,
    Skimmer,
    Blend,
}

impl System {
    pub fn new() -> Self {
        System {
            system_mode: SystemMode::Pool,
            pool_mode: PoolMode::Skimmer,
            filter_on: false,
            heater_on: false,
            jets_on: false,
            valves_changing: false,
            errors: [None; 10],
        }
    }
    // JETS
    pub fn run_jets(&mut self) {
        self.jets_on = true;
    }
    pub fn stop_jets(&mut self) {
        self.jets_on = false;
    }

    // FILTER
    pub fn run_filter(&mut self) {
        self.filter_on = true;
    }
    pub fn stop_filter(&mut self) {
        self.filter_on = false;
    }

    // POOL MODE
    pub fn change_pool_mode(&mut self, mode: PoolMode) {
        self.pool_mode = mode;
    }

    // HEATER
    pub fn run_heater(&mut self) {
        self.heater_on = true;
    }
    pub fn cancel_heater(&mut self) {
        self.heater_on = false;
    }
}
