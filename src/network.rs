use feather_m0::prelude::*;
use radio_sx127x::prelude::*;

// TOODO: get this from radio_sx127x
use embedded_spi::wrapper::{Wrapper as SpiWrapper};

// use feather_m0::cortex_m::hal

pub struct Radio<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> {
    radio: Sx127x<SpiWrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>, SpiError, PinError>,
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
    Radio<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Spi: _embedded_hal_blocking_spi_Transfer<u8, Error = SpiError> + _embedded_hal_blocking_spi_Write<u8, Error = SpiError>,
    CsPin: _atsamd_hal_embedded_hal_digital_v2_OutputPin<Error = PinError>,
    BusyPin: _atsamd_hal_embedded_hal_digital_v2_InputPin<Error = PinError>,
    // TODO: should ReadyPin have a where?
    ResetPin: _atsamd_hal_embedded_hal_digital_v2_OutputPin<Error = PinError>,
    Delay: _embedded_hal_blocking_delay_DelayMs<u32>,
{
    pub fn new(
        spi: Spi,
        cs: CsPin,
        busy: BusyPin,
        ready: ReadyPin,
        reset: ResetPin,
        delay: Delay,
    ) -> Self {
        // TODO: what config?
        let config = Config::default();

        let radio = Sx127x::spi(spi, cs, busy, ready, reset, delay, &config).unwrap_or_else(|_| panic!());

        Self {
            radio
        }
    }
}