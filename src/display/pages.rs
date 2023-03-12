use std::fmt::Debug;

use embedded_graphics::{
    image::Image,
    mono_font::{ascii::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};

use crate::display::icons::*;
use crate::display::widgets::*;
use crate::driver::linky::TariffPeriod;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Page {
    Landing,
    Rpict,
    Linky,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LandingPage {
    is_rpict_on: bool,
    is_linky_on: bool,
    is_wifi_on: bool,
    version: String
}

impl LandingPage {
    pub fn new(version: &str) -> Self {
        let version = version.to_string();
        Self {
            is_rpict_on: false,
            is_linky_on: false,
            is_wifi_on: false,
            version,
        }
    }

    pub fn rpict_status(&mut self, is_running: bool) {
        self.is_rpict_on = is_running;
    }

    pub fn linky_status(&mut self, is_running: bool) {
        self.is_linky_on = is_running;
    }

    pub fn wifi_status(&mut self, is_running: bool) {
        self.is_wifi_on = is_running;
    }
}

impl Drawable for LandingPage {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where D: DrawTarget<Color=Self::Color> {
        target.clear(BinaryColor::Off)?;

        const CENTER: Point = Point::new(128 / 2, 32 / 2);
        const LOGO_OFFSET: Point = Point::new(0, -4);
        const VERSION_OFFSET: Point = Point::new(46, 10);

        Rectangle::with_center(CENTER + LOGO_OFFSET - Point::new(0, 2), Size::new(90, 15))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(target)?;

        Text::with_alignment(
            "energy-monitor",
            CENTER + LOGO_OFFSET,
            MonoTextStyle::new(&FONT_6X12, BinaryColor::On),
            Alignment::Center,
        ).draw(target)?;

        let rpict_icon = if self.is_rpict_on { &*RPICT_ON } else { &*RPICT_OFF };
        Image::new(rpict_icon, Point::new(20, 20))
            .draw(target)?;

        let linky_icon = if self.is_linky_on { &*LINKY_ON } else { &*LINKY_OFF };
        Image::new(linky_icon, Point::new(30, 20))
            .draw(target)?;

        let wifi_icon = if self.is_wifi_on { &*WIFI_ON } else { &*WIFI_OFF };
        Image::new(wifi_icon, Point::new(40, 20))
            .draw(target)?;

        Text::with_alignment(
            &format!("v{}", self.version),
            CENTER + VERSION_OFFSET,
            MonoTextStyle::new(&FONT_4X6, BinaryColor::On),
            Alignment::Right,
        ).draw(target)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RpictPage {
    l1_sparkline: SparkLine,
    l2_sparkline: SparkLine,
    l3_sparkline: SparkLine,
    total_apparent_power: f32,
    avg_vrms: f32,
}

impl RpictPage {
    pub fn new(max_power: f32) -> Self {
        Self {
            l1_sparkline: SparkLine::new(Point::new(1, 6), "P1".to_string(), max_power),
            l2_sparkline: SparkLine::new(Point::new(1, 14), "P2".to_string(), max_power),
            l3_sparkline: SparkLine::new(Point::new(1, 22), "P3".to_string(), max_power),
            total_apparent_power: 0.0,
            avg_vrms: 0.0,
        }
    }

    pub fn update(&mut self,
                  l1_apparent_power: f32,
                  l2_apparent_power: f32,
                  l3_apparent_power: f32,
                  l1_vrms: f32,
                  l2_vrms: f32,
                  l3_vrms: f32) {
        self.l1_sparkline.update(l1_apparent_power);
        self.l2_sparkline.update(l2_apparent_power);
        self.l3_sparkline.update(l3_apparent_power);
        self.total_apparent_power = l1_apparent_power + l2_apparent_power + l3_apparent_power;
        self.avg_vrms = (l1_vrms + l2_vrms + l3_vrms) / 3.0;
    }
}

impl Drawable for RpictPage {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where D: DrawTarget<Color=Self::Color> {
        target.clear(BinaryColor::Off)?;

        self.l1_sparkline.draw(target)?;
        self.l2_sparkline.draw(target)?;
        self.l3_sparkline.draw(target)?;

        let text_style = MonoTextStyle::new(&FONT_5X7, BinaryColor::On);

        Text::with_alignment(
            &format!("= {:4.1}kW", self.total_apparent_power / 1000.0),
            Point::new(1, 30),
            text_style,
            Alignment::Left,
        ).draw(target)?;

        Text::with_alignment(
            &format!("{:5.2}V", self.avg_vrms),
            Point::new(127, 30),
            text_style,
            Alignment::Right,
        ).draw(target)?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinkyPage {
    adco: String,
    hchp: Option<u32>,
    hchc: Option<u32>,
    ptec: TariffPeriod,
}

impl LinkyPage {
    pub fn new() -> Self {
        Self {
            adco: "?".to_string(),
            hchp: None,
            hchc: None,
            ptec: TariffPeriod::Unknown,
        }
    }

    pub fn update(&mut self,
                  adco: String,
                  hchp: u32,
                  hchc: u32,
                  ptec: TariffPeriod) {
        self.adco = adco;
        self.hchp = Some(hchp);
        self.hchc = Some(hchc);
        self.ptec = ptec;
    }
}

impl Drawable for LinkyPage {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where D: DrawTarget<Color=Self::Color> {
        target.clear(BinaryColor::Off)?;

        const FORMAT: fn(u32) -> String = |x| format!("{:9.3}kWh", x as f32 / 1000.0);
        const UNKNOWN: fn() -> String = || "?".to_string();
        let text_style = MonoTextStyle::new(&FONT_5X7, BinaryColor::On);

        Text::with_alignment(
            &format!("ID {}", &self.adco),
            Point::new(64, 8),
            text_style,
            Alignment::Center,
        ).draw(target)?;

        let hchp_active = match self.ptec { TariffPeriod::HP => ">", _ => " " };
        let hchp = self.hchp.map(FORMAT).unwrap_or_else(UNKNOWN);
        let hchc_active = match self.ptec { TariffPeriod::HC => ">", _ => " " };
        let hchc = self.hchc.map(FORMAT).unwrap_or_else(UNKNOWN);
        Text::with_alignment(
            &format!("{}HP {}\n{}HC {}", hchp_active, hchp, hchc_active, hchc),
            Point::new(64, 20),
            text_style,
            Alignment::Center,
        ).draw(target)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_landingpage_new() {
        // When
        let actual = LandingPage::new("0.0.0");
        // Then
        assert!(matches!(actual, LandingPage { is_rpict_on: false, is_linky_on: false, is_wifi_on: false, version }
            if version == "0.0.0"));
    }

    #[test]
    fn test_landingpage_update() {
        // Given
        let mut actual = LandingPage::new("0.0.0");
        // When
        actual.rpict_status(true);
        actual.linky_status(true);
        actual.wifi_status(true);
        // Then
        assert!(matches!(actual, LandingPage { is_rpict_on: true, is_linky_on: true, is_wifi_on: true, .. }));
    }

    #[test]
    fn test_rpictpage_new() {
        // When
        let actual = RpictPage::new(8000.0);
        // Then
        assert!(matches!(actual, RpictPage { total_apparent_power, avg_vrms, .. }
            if total_apparent_power == 0.0
            && avg_vrms == 0.0));
    }

    #[test]
    fn test_rpictpage_update() {
        // Given
        let mut actual = RpictPage::new(8000.0);
        // When
        actual.update(100.0,
                      200.0,
                      300.0,
                      222.0,
                      224.0,
                      226.0);
        // Then
        assert!(matches!(actual, RpictPage { total_apparent_power, avg_vrms, .. }
            if total_apparent_power == 600.0
            && avg_vrms == 224.0));
    }

    #[test]
    fn test_linkypage_new() {
        // When
        let actual = LinkyPage::new();
        // Then
        assert!(matches!(actual, LinkyPage { adco, hchp: None, hchc: None, ptec: TariffPeriod::Unknown }
            if adco == "?"));
    }

    #[test]
    fn test_linkypage_update() {
        // Given
        let mut actual = LinkyPage::new();
        // When
        actual.update("1234".into(), 11, 22, TariffPeriod::HC);
        // Then
        assert!(matches!(actual, LinkyPage { adco, hchp: Some(11), hchc: Some(22), ptec: TariffPeriod::HC }
            if adco == "1234"));
    }
}
