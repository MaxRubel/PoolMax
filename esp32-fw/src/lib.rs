// embedded/src/hardware.rs or embedded/src/main.rs
use app_core::{System, PoolMode};
use esp32_hal::gpio::{GpioPin, Output, PushPull};

pub struct HardwareController {
    system: System,
    
    // Hardware pins
    filter_relay: GpioPin<Output<PushPull>, 2>,
    heater_relay: GpioPin<Output<PushPull>, 4>,
    jets_relay: GpioPin<Output<PushPull>, 5>,
    valve_motor_pwm: /* whatever your valve motor needs */,
    
    // Tracking for async operations
    valve_move_start_time: Option<u32>, // timestamp when valve started moving
}

impl HardwareController {
    pub fn new(
        filter_pin: GpioPin<Output<PushPull>, 2>,
        heater_pin: GpioPin<Output<PushPull>, 4>,
        jets_pin: GpioPin<Output<PushPull>, 5>,
        // ... other pins
    ) -> Self {
        HardwareController {
            system: System::new(),
            filter_relay: filter_pin,
            heater_relay: heater_pin,
            jets_relay: jets_pin,
            valve_motor_pwm: /* ... */,
            valve_move_start_time: None,
        }
    }
    
    // Hardware-aware methods that update state AND control hardware
    
    pub fn toggle_jets(&mut self) {
        self.system.toggle_jets();
        self.sync_jets_hardware();
    }
    
    pub fn toggle_filter(&mut self) {
        let was_on = self.system.filter_on;
        self.system.toggle_filter();
        
        // Only sync if state actually changed (valve might prevent it)
        if was_on != self.system.filter_on {
            self.sync_filter_hardware();
        }
    }
    
    pub fn toggle_heater(&mut self) {
        self.system.toggle_heater();
        self.sync_heater_hardware();
    }
    
    pub fn change_pool_mode(&mut self, mode: PoolMode) {
        if self.system.valves_changing {
            return; // Already moving
        }
        
        let filter_was_on = self.system.filter_on;
        
        // Update state (this turns off filter if needed)
        self.system.start_valve_change(mode);
        
        // Sync hardware
        if filter_was_on {
            self.sync_filter_hardware(); // Turn off physically
        }
        
        self.move_valves_to(mode);
        self.valve_move_start_time = Some(get_current_millis());
    }
    
    pub fn check_valve_completion(&mut self) {
        if let Some(start_time) = self.valve_move_start_time {
            let elapsed = get_current_millis() - start_time;
            
            // Valves take ~5 seconds to move
            if elapsed > 5000 {
                self.valve_move_start_time = None;
                self.system.complete_valve_change();
                
                // Note: filter stays off, user must manually turn back on
                // Or auto-restart if it was on:
                // if self.system.filter_on {
                //     self.sync_filter_hardware();
                // }
            }
        }
    }
    
    // Private hardware sync methods
    
    fn sync_filter_hardware(&mut self) {
        if self.system.filter_on {
            self.filter_relay.set_high();
            println!("HW: Filter relay ON");
        } else {
            self.filter_relay.set_low();
            println!("HW: Filter relay OFF");
        }
    }
    
    fn sync_heater_hardware(&mut self) {
        if self.system.heater_on {
            self.heater_relay.set_high();
            println!("HW: Heater relay ON");
        } else {
            self.heater_relay.set_low();
            println!("HW: Heater relay OFF");
        }
    }
    
    fn sync_jets_hardware(&mut self) {
        if self.system.jets_on {
            self.jets_relay.set_high();
            println!("HW: Jets relay ON");
        } else {
            self.jets_relay.set_low();
            println!("HW: Jets relay OFF");
        }
    }
    
    fn move_valves_to(&mut self, mode: PoolMode) {
        // Control your valve motor based on mode
        match mode {
            PoolMode::Vacuum => {
                // Set PWM or whatever to move to vacuum position
                println!("HW: Moving valves to VACUUM");
            }
            PoolMode::Skimmer => {
                println!("HW: Moving valves to SKIMMER");
            }
            PoolMode::Blend => {
                println!("HW: Moving valves to BLEND");
            }
        }
    }
    
    // Direct access to state for reading
    pub fn get_state(&self) -> &System {
        &self.system
    }
}