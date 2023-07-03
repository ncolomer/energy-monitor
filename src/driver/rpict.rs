use std::error::Error;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use rppal::uart::{Parity, Uart};

use crate::driver::error::ParseError;

#[derive(Clone, Debug, PartialEq)]
pub struct RpictFrame {
    pub node_id: u8,
    pub l1_real_power: f32,
    pub l1_apparent_power: f32,
    pub l1_irms: f32,
    pub l1_vrms: f32,
    pub l1_power_factor: f32,
    pub l2_real_power: f32,
    pub l2_apparent_power: f32,
    pub l2_irms: f32,
    pub l2_vrms: f32,
    pub l2_power_factor: f32,
    pub l3_real_power: f32,
    pub l3_apparent_power: f32,
    pub l3_irms: f32,
    pub l3_vrms: f32,
    pub l3_power_factor: f32,
    pub timestamp: DateTime<Utc>,
}

impl RpictFrame {
    fn parse(input: &str, dt_gen: &dyn Fn() -> DateTime<Utc>) -> Result<Self, ParseError> {
        fn consume<'a, T: FromStr>(iter: &mut impl Iterator<Item = &'a str>) -> Result<T, ParseError> {
            iter.next().ok_or(ParseError)?.parse().or(Err(ParseError))
        }
        let mut split = input.split_ascii_whitespace().fuse();
        let frame = RpictFrame {
            node_id: consume(&mut split)?,
            l1_real_power: consume(&mut split)?,
            l1_apparent_power: consume(&mut split)?,
            l1_irms: consume(&mut split)?,
            l1_vrms: consume(&mut split)?,
            l1_power_factor: consume(&mut split)?,
            l2_real_power: consume(&mut split)?,
            l2_apparent_power: consume(&mut split)?,
            l2_irms: consume(&mut split)?,
            l2_vrms: consume(&mut split)?,
            l2_power_factor: consume(&mut split)?,
            l3_real_power: consume(&mut split)?,
            l3_apparent_power: consume(&mut split)?,
            l3_irms: consume(&mut split)?,
            l3_vrms: consume(&mut split)?,
            l3_power_factor: consume(&mut split)?,
            timestamp: dt_gen(),
        };
        match split.next() {
            None => Ok(frame),
            Some(_) => Err(ParseError),
        }
    }
}

struct RpictIterator {
    uart: Uart,
    buffer: [u8; 1],
}

impl Iterator for RpictIterator {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.uart.read(&mut self.buffer) {
                Ok(size) if size > 0 => return Some(self.buffer[0].into()),
                Ok(_size) => continue,
                Err(_) => {
                    log::error!("error while reading bytestream");
                    return None;
                }
            }
        }
    }
}

pub struct Rpict {
    port_path: Option<String>,
    source_iter: Option<Box<dyn Iterator<Item = char>>>,
    dt_gen: Rc<dyn Fn() -> DateTime<Utc>>,
}

impl Rpict {
    pub fn builder() -> Self {
        Self {
            port_path: None,
            source_iter: None,
            dt_gen: Rc::new(Utc::now),
        }
    }

    pub fn with_port_path(mut self, port_path: String) -> Self {
        self.port_path = Some(port_path);
        self
    }

    pub fn with_source_iter(mut self, source_iter: impl Iterator<Item = char> + 'static) -> Self {
        self.source_iter = Some(Box::new(source_iter));
        self
    }

    pub fn with_dt_gen(mut self, dt_gen: impl Fn() -> DateTime<Utc> + 'static) -> Self {
        self.dt_gen = Rc::new(dt_gen);
        self
    }

    pub fn build(self) -> Result<impl Iterator<Item = RpictFrame>, Box<dyn Error>> {
        let Self {
            port_path,
            source_iter,
            dt_gen,
        } = self;
        let source_iter = source_iter
            .or_else(|| {
                let port_path = port_path.expect("no port path provided");
                let mut uart = Uart::with_path(Path::new(&port_path), 38_400, Parity::None, 8, 1).unwrap();
                uart.set_read_mode(1, Duration::default()).unwrap();
                Some(Box::new(RpictIterator { uart, buffer: [0u8] }))
            })
            .expect("no source provided");
        let iter = source_iter
            //.skip_while(|c| future::ready(c != '\n'))
            .scan(String::new(), |buffer, c| {
                if c == '\n' {
                    let line = buffer.clone();
                    buffer.clear();
                    Some(Some(line))
                } else {
                    buffer.push(c);
                    Some(None)
                }
            })
            .flatten()
            .filter_map(move |s| match RpictFrame::parse(&s, &*dt_gen.clone()) {
                Ok(frame) => Some(frame),
                Err(_) => {
                    log::warn!("couldn't extract RpictFrame from string {}", s);
                    None
                }
            });
        Ok(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: &str =
        "11 -82.96 422.95 1.64 257.65 0.194 -50.23 144.52 0.56 259.95 0.346 24.55 47.17 0.18 259.70 0.509";
    fn frame(now: DateTime<Utc>) -> RpictFrame {
        RpictFrame {
            node_id: 11,
            l1_real_power: -82.96,
            l1_apparent_power: 422.95,
            l1_irms: 1.64,
            l1_vrms: 257.65,
            l1_power_factor: 0.194,
            l2_real_power: -50.23,
            l2_apparent_power: 144.52,
            l2_irms: 0.56,
            l2_vrms: 259.95,
            l2_power_factor: 0.346,
            l3_real_power: 24.55,
            l3_apparent_power: 47.17,
            l3_irms: 0.18,
            l3_vrms: 259.70,
            l3_power_factor: 0.509,
            timestamp: now,
        }
    }

    #[test]
    fn test_rpictiterator() {
        // Given
        let now = Utc::now();
        let input = FRAME.chars().chain(vec!['\n']);
        // When
        let frames = Rpict::builder()
            .with_source_iter(input)
            .with_dt_gen(move || now)
            .build()
            .unwrap();
        // Then
        assert_eq!(frames.collect::<Vec<RpictFrame>>(), vec![frame(now)]);
    }

    #[test]
    fn test_rpictiterator_with_two_frames() {
        // Given
        let now = Utc::now();
        let input = FRAME.chars().chain(vec!['\n']);
        let input_len = FRAME.len() + 1;
        // When
        let frames = Rpict::builder()
            .with_source_iter(input.clone().cycle().take(input_len * 2))
            .with_dt_gen(move || now)
            .build()
            .unwrap();
        // Then
        assert_eq!(frames.collect::<Vec<RpictFrame>>(), vec![frame(now), frame(now)]);
    }

    #[test]
    fn test_rpictiterator_with_corrupted_frame() {
        // Given
        let now = Utc::now();
        let input = FRAME[..10].chars().chain(vec!['\n']);
        // When
        let frames = Rpict::builder()
            .with_source_iter(input)
            .with_dt_gen(move || now)
            .build()
            .unwrap();
        // Then
        assert_eq!(frames.collect::<Vec<RpictFrame>>(), Vec::new());
    }

    #[test]
    fn test_rpictframe_parse() {
        // Given
        let now = Utc::now();
        // When
        let result = RpictFrame::parse(FRAME, &|| now).unwrap();
        // Then
        assert_eq!(result, frame(now));
    }

    #[test]
    fn test_rpictframe_parse_not_enough_data_missing_start() {
        // Given
        let input = &FRAME[10..];
        // When
        let result = RpictFrame::parse(input, &Utc::now);
        // Then
        assert_eq!(result, Err(ParseError))
    }

    #[test]
    fn test_rpictframe_parse_not_enough_data_missing_end() {
        // Given
        let input = &FRAME[..10];
        // When
        let result = RpictFrame::parse(input, &Utc::now);
        // Then
        assert_eq!(result, Err(ParseError))
    }

    #[test]
    fn test_rpictframe_parse_too_much_data() {
        // Given
        let input = FRAME.to_string() + " 259.70 0.509";
        // When
        let result = RpictFrame::parse(&input, &Utc::now);
        // Then
        assert_eq!(result, Err(ParseError))
    }
}
