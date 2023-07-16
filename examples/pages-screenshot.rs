
use std::path::Path;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::BinaryColor;

use image::{ImageBuffer, Luma, imageops::resize};
use energy_monitor::display::pages::*;
use energy_monitor::display::icons::*;
use energy_monitor::driver::linky::TariffPeriod;
use energy_monitor::driver::ssd1305::{DISPLAY_WIDTH, DISPLAY_HEIGHT};

// Inspired from https://github.com/embedded-graphics/embedded-graphics/blob/657fb4b/tools/png-target/src/lib.rs

pub struct PngTarget {
    image: ImageBuffer<Luma<u8>, Vec<u8>>,
}

impl PngTarget {
    pub fn new(size: Size) -> Self {
        Self {
            image: ImageBuffer::new(size.width, size.height),
        }
    }

    pub fn save<PATH: AsRef<Path>>(&self, path: PATH) -> image::ImageResult<()> {
        let scale = 3;
        resize(
            &self.image,
            self.image.width() * scale,
            self.image.height() * scale,
            image::imageops::FilterType::Nearest,
        ).save_with_format(path, image::ImageFormat::Png)
    }

    pub fn save_page<D, P: AsRef<Path>>(&mut self, drawable: &D, path: P)
        where D: Drawable<Color=BinaryColor> {
        drawable.draw(self).unwrap();
        self.save(path).unwrap();
    }

    pub fn save_image<P: AsRef<Path>>(&mut self, drawable: &ImageRaw<BinaryColor>, path: P) {
        drawable.draw(self).unwrap();
        self.save(path).unwrap();
    }
}

impl DrawTarget for PngTarget {
    type Color = BinaryColor;
    type Error = std::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item=Pixel<Self::Color>>,
    {
        for Pixel(p, c) in pixels {
            if let (Ok(x), Ok(y)) = (u32::try_from(p.x), u32::try_from(p.y)) {
                if x < self.image.width() && y < self.image.height() {
                    self.image.put_pixel(
                        p.x as u32,
                        p.y as u32,
                        Luma([if c.is_on() { 255 } else { 0 }]),
                    );
                }
            }
        }

        Ok(())
    }
}

impl OriginDimensions for PngTarget {
    fn size(&self) -> Size {
        Size::new(self.image.width(), self.image.height())
    }
}

fn main() {
    // Save pages
    let mut display = PngTarget::new(Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32));

    let mut page = StartupPage::new("1.2.3");
    page.rpict_status(true);
    page.linky_status(true);
    page.influxdb_status(true);
    display.save_page(&page, Path::new("page-startup.png"));

    let mut page = RpictPage::new(8000.0);
    page.update(3076.0, 2229.0, 6403.0, 232.0, 232.0, 232.0);
    page.update(232.0, 1540.0, 5670.0, 232.0, 232.0, 232.0);
    display.save_page(&page, Path::new("page-rpict.png"));

    let mut page = LinkyPage::new();
    page.update("005215329642".to_string(), 22_965_852, 7_431_234, TariffPeriod::HP);
    display.save_page(&page, Path::new("page-linky.png"));

    // Save icons
    let mut display = PngTarget::new(Size::new(8, 8));

    display.save_image(&RPICT_ON, Path::new("icon-rpict-on.png"));
    display.save_image(&RPICT_OFF, Path::new("icon-rpict-off.png"));
    display.save_image(&LINKY_ON, Path::new("icon-linky-on.png"));
    display.save_image(&LINKY_OFF, Path::new("icon-linky-off.png"));
    display.save_image(&INFLUXDB_ON, Path::new("icon-influxdb-on.png"));
    display.save_image(&INFLUXDB_OFF, Path::new("icon-influxdb-off.png"));
}
