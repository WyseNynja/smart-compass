#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

// panic handler
use panic_halt as _;

pub extern crate feather_m0 as hal;

use hal::prelude::*;

use alloc_cortex_m::CortexMHeap;
use asm_delay::AsmDelay;
use core::alloc::Layout;
use hal::clock::GenericClockController;
use rtic::app;
use smart_compass::{lights, periodic, ELAPSED_MS};

// TODO: i'm not sure what I did to require an allocator
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

pub type SPIMaster = hal::sercom::SPIMaster4<
    hal::sercom::Sercom4Pad0<hal::gpio::Pa12<hal::gpio::PfD>>,
    hal::sercom::Sercom4Pad2<hal::gpio::Pb10<hal::gpio::PfD>>,
    hal::sercom::Sercom4Pad3<hal::gpio::Pb11<hal::gpio::PfD>>,
>;

// static globals
// the number of ms to offset our network timer. this is time to send+receive+process+draw
// static NETWORK_OFFSET: u16 = 125 + 225;
static DEFAULT_BRIGHTNESS: u8 = 128;
static FRAMES_PER_SECOND: u8 = 30;

#[app(device = hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        lights: lights::Lights<SPIMaster>,
        every_500_millis: periodic::Periodic,
        red_led: hal::gpio::Pa17<hal::gpio::Output<hal::gpio::OpenDrain>>,
        timer4: hal::timer::TimerCounter4,
    }

    /// This function is called each time the tc4 interrupt triggers.
    /// We use it to toggle the LED.  The `wait()` call is important
    /// because it checks and resets the counter ready for the next
    /// period.
    /// TODO: is this how arduino's millis function works?
    #[task(binds = TC4, resources = [timer4], priority = 3)]
    fn tc4(c: tc4::Context) {
        if c.resources.timer4.wait().is_ok() {
            unsafe {
                ELAPSED_MS += 1;
            }
        }
    }

    /// setup the hardware
    #[init]
    fn init(c: init::Context) -> init::LateResources {
        // Initialize the allocator BEFORE you use it
        let start = cortex_m_rt::heap_start() as usize;
        let size = 1024; // in bytes
        unsafe { ALLOCATOR.init(start, size) }

        let mut device = c.device;

        let mut clocks = GenericClockController::with_internal_32kosc(
            device.GCLK,
            &mut device.PM,
            &mut device.SYSCTRL,
            &mut device.NVMCTRL,
        );
        let gclk0 = clocks.gclk0();
        let mut pins = hal::Pins::new(device.PORT);

        // TODO: which timer should we use?
        let mut timer4 = hal::timer::TimerCounter::tc4_(
            &clocks.tc4_tc5(&gclk0).unwrap(),
            device.TC4,
            &mut device.PM,
        );

        // the ws2812-spi library says between 2-3.8 or something like that
        let my_spi = hal::spi_master(
            &mut clocks,
            3.mhz(),
            device.SERCOM4,
            &mut device.PM,
            pins.sck,
            pins.mosi,
            pins.miso,
            &mut pins.port,
        );

        // onboard LED
        let red_led = pins.d13.into_open_drain_output(&mut pins.port);

        // external LEDs
        let my_lights = lights::Lights::new(my_spi, DEFAULT_BRIGHTNESS, FRAMES_PER_SECOND);

        // TODO: setup USB serial for debug logging

        let every_500_millis = periodic::Periodic::new(500);

        // timer for ELAPSED_MILLIS
        // TODO: i am not positive that this is correct. every example seems to do timers differently
        // the feather_m0 runs at 48 MHz. so 48 MHz / 1000 milliseconds = MHz / millisecond
        timer4.start(48.khz());
        timer4.enable_interrupt();

        init::LateResources {
            every_500_millis,
            lights: my_lights,
            red_led,
            timer4,
        }
    }

    // `shared` cannot be accessed from this context
    // TODO: more of this should probably be done with interrupts
    #[idle(resources = [
        // every_500_millis,
        // lights,
        red_led,
    ])]
    fn idle(c: idle::Context) -> ! {
        // let my_lights = c.resources.lights;
        // let every_500_millis = c.resources.every_500_millis;
        let red_led = c.resources.red_led;

        let mut delay = AsmDelay::new(asm_delay::bitrate::U32BitrateExt::mhz(48));

        // my_lights.draw_test_pattern();

        loop {
            // TODO: change pattern every few seconds?
            // if every_500_millis.ready() {
            //     red_led.toggle();
            // }

            red_led.set_high().unwrap();
            delay.delay_ms(200u8);
            red_led.set_low().unwrap();
            delay.delay_ms(200u8);

            // my_lights.draw();
        }
    }
};

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {}
}
