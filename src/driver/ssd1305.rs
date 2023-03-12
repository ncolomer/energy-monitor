use std::error::Error;
use std::thread;
use std::time::Duration;

use embedded_graphics::{
    prelude::*,
};
use embedded_graphics::pixelcolor::BinaryColor;
use rppal::gpio;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

use crate::driver::error::CommError;

const GPIO_DC: u8 = 24;
const GPIO_RST: u8 = 25;
const DISPLAY_WIDTH: usize = 128;
const DISPLAY_HEIGHT: usize = 32;

const SET_DISP: u8 = 0xAE; // turn display on/off
const SET_CONTRAST: u8 = 0x81; // set display contrast

pub struct Ssd1305 {
    buffer: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT / 8],
    gpio_dc: gpio::OutputPin,
    gpio_rst: gpio::OutputPin,
    spi: Spi,
}

impl Ssd1305 {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let buffer = [0x00; DISPLAY_WIDTH * DISPLAY_HEIGHT / 8];
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 8_000_000, Mode::Mode0)?;
        let gpio = gpio::Gpio::new()?;
        let gpio_dc = gpio.get(GPIO_DC)?.into_output();
        let gpio_rst = gpio.get(GPIO_RST)?.into_output();
        Ok(Self { buffer, gpio_dc, gpio_rst, spi })
    }

    fn reset(&mut self) {
        const DURATION: Duration = Duration::from_millis(10);
        self.gpio_rst.write(gpio::Level::High);
        thread::sleep(DURATION);
        self.gpio_rst.write(gpio::Level::Low);
        thread::sleep(DURATION);
        self.gpio_rst.write(gpio::Level::High);
    }

    fn command(&mut self, cmd: u8) -> Result<(), CommError> {
        self.gpio_dc.write(gpio::Level::Low);
        self.spi.write(&[cmd]).map_err(|_|CommError)?;
        Ok(())
    }

    pub fn begin(&mut self) -> Result<(), CommError> {
        self.reset();
        self.display_off()?;
        self.command(0x04)?; //--Set Lower Column Start Address for Page Addressing Mode
        self.command(0x10)?; //--Set Higher Column Start Address for Page Addressing Mode
        self.command(0x40)?; //---Set Display Start Line
        self.display_contrast(0x00)?;
        self.command(0xA1)?; //--Set Segment Re-map
        self.command(0xA6)?; //--Set Normal/Inverse Display
        self.command(0xA8)?; //--Set Multiplex Ratio
        self.command(0x1F)?;
        self.command(0xC8)?; //--Set COM Output Scan Direction
        self.command(0xD3)?; //--Set Display Offset
        self.command(0x00)?;
        self.command(0xD5)?; //--Set Display Clock Divide Ratio/ Oscillator Frequency
        self.command(0xF0)?;
        self.command(0xD8)?; //--Set Area Color Mode ON/OFF & Low Power Display Mode
        self.command(0x05)?;
        self.command(0xD9)?; //--Set pre-charge period
        self.command(0xC2)?;
        self.command(0xDA)?; //--Set COM Pins Hardware Configuration
        self.command(0x12)?;
        self.command(0xDB)?; //--Set VCOMH Deselect Level
        self.command(0x08)?; //--Set VCOM Deselect Level
        Ok(())
    }

    pub fn display_on(&mut self) -> Result<(), CommError> {
        self.command(SET_DISP | 0x01)?;
        Ok(())
    }

    pub fn display_off(&mut self) -> Result<(), CommError> {
        self.command(SET_DISP | 0x00)?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), CommError> {
        self.buffer.fill(0x00);
        self.flush()?;
        Ok(())
    }

    fn display_contrast(&mut self, value: u8) -> Result<(), CommError> {
        self.command(SET_CONTRAST)?;
        self.command(value)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), CommError> {
        for page in 0..4 {
            self.command(0xB0 + page)?; // Set page address
            self.command(0x04)?; // Set low column address
            self.command(0x10)?; // Set high column address
            self.gpio_dc.write(gpio::Level::High);

            let page_usize = page as usize;
            let start_index: usize = page_usize * DISPLAY_WIDTH;
            let end_index: usize = start_index + DISPLAY_WIDTH;
            let page_slice = &self.buffer[start_index..end_index];

            self.spi.write(page_slice).map_err(|_|CommError)?;
        }
        Ok(())
    }
}

impl DrawTarget for Ssd1305 {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x, y }, color) in pixels.into_iter() {
            if x < 0 || y < 0 || x >= DISPLAY_WIDTH as i32 || y >= DISPLAY_HEIGHT as i32 { continue; }
            let (x, y) = (x as usize, y as usize);
            let index = x + (y / 8) * DISPLAY_WIDTH;
            match color {
                BinaryColor::On => self.buffer[index] |= 1 << (y % 8),
                BinaryColor::Off => self.buffer[index] &= !(1 << (y % 8))
            }
        }
        Ok(())
    }
}

impl OriginDimensions for Ssd1305 {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
    }
}
