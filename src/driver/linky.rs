use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use rppal::uart::{Parity, Uart};

use crate::driver::error::ParseError;

#[derive(Clone, Debug, PartialEq)]
pub struct LinkyFrame {
    pub adco: String, // electric meter address
    pub ptec: String, // current tariff period
    pub hchc: u32, // heures creuses index, in watts
    pub hchp: u32, // heures pleines index, in watts
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TariffPeriod {
    HC, HP, Unknown
}

impl LinkyFrame {

    fn parse(map: &HashMap<String, String>, dt_gen: &dyn Fn() -> DateTime<Utc>) -> Result<Self, ParseError> {
        fn extract<T: FromStr>(option: Option<&String>) -> Result<T, ParseError> {
            option.ok_or(ParseError)?.parse().or(Err(ParseError))
        }
        let frame = LinkyFrame {
            adco: extract(map.get("ADCO"))?,
            ptec: extract(map.get("PTEC"))?,
            hchc: extract(map.get("HCHC"))?,
            hchp: extract(map.get("HCHP"))?,
            timestamp: dt_gen(),
        };
        Ok(frame)
    }

    pub fn ptec(&self) -> TariffPeriod {
        match self.ptec.as_str() {
            "HC" => TariffPeriod::HC,
            "HP" => TariffPeriod::HP,
            _ => TariffPeriod::Unknown,
        }
    }

}

struct LinkyIterator {
    uart: Uart,
    buffer: [u8; 1]
}

impl Iterator for LinkyIterator {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.uart.read(&mut self.buffer) {
                Ok(size) if size > 0 =>
                    return Some(self.buffer[0].into()),
                Ok(_size) =>
                    continue,
                Err(_) => {
                    log::error!("error while reading bytestream");
                    return None
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum Frame {
    Start,
    DataSet(String),
    End,
}

pub struct Linky {
    port_path: Option<String>,
    source_iter: Option<Box<dyn Iterator<Item=char>>>,
    // Using Rc because Box is not easily Copy-able
    // See https://users.rust-lang.org/t/how-to-clone-a-boxed-closure/31035
    dt_gen: Rc<dyn Fn() -> DateTime<Utc>>
}

impl Linky {

    pub fn builder() -> Self {
        Self {
            port_path :None,
            source_iter: None,
            dt_gen: Rc::new(Utc::now)
        }
    }

    pub fn with_port_path(mut self, port_path: String) -> Self {
        self.port_path = Some(port_path);
        self
    }

    pub fn with_source_iter(mut self, source_iter: impl Iterator<Item=char> + 'static) -> Self {
        self.source_iter = Some(Box::new(source_iter));
        self
    }

    pub fn with_dt_gen(mut self, dt_gen: impl Fn() -> DateTime<Utc> + 'static) -> Self{
        self.dt_gen = Rc::new(dt_gen);
        self
    }

    pub fn build(self) -> Result<impl Iterator<Item=LinkyFrame>, Box<dyn Error>> {
        const STX: char = '\u{02}'; // frame start
        const ETX: char = '\u{03}'; // frame end
        const LF: char = '\u{0A}'; // group start
        const CR: char = '\u{0D}'; // group end
        const KEYS: [&str; 4] = ["ADCO", "PTEC", "HCHC", "HCHP"];
        let Self { port_path, source_iter, dt_gen } = self;
        let source_iter = source_iter.or_else(|| {
            let port_path = port_path.expect("no port path provided");
            // See https://www.enedis.fr/sites/default/files/Enedis-NOI-CPT_54E.pdf
            // 5.3.5. Couche physique — Page : 12/38
            let mut uart = Uart::with_path(
                Path::new(&port_path),
                1_200,
                Parity::Even,
                7,
                1
            ).unwrap();
            uart.set_read_mode(1, Duration::default()).unwrap();
            Some(Box::new(LinkyIterator {uart, buffer: [0u8]}))
        }).expect("no source provided");
        // TIC mode Historique (vs. new Standard mode)
        // < LF > (0x0A) | Etiquette | < HT > (0x09) | Donnée | < HT > (0x09) | Checksum | < CR > (0x0D)
        // See https://www.enedis.fr/sites/default/files/Enedis-NOI-CPT_54E.pdf
        // 5.3.6. Couche liaison — Page : 13/38
        let iter =
            source_iter
                //.skip_while(|c| future::ready(c != '\n'))
                .scan(String::new(), |buffer, c| {
                    match c {
                        STX => Some(Some(Frame::Start)),
                        ETX => Some(Some(Frame::End)),
                        LF => { buffer.clear(); Some(None) }
                        CR => { Some(Some(Frame::DataSet(buffer.clone()))) }
                        other => { buffer.push(other); Some(None) }
                    }
                })
                .flatten()
                .scan(HashMap::<String, String>::new(), |buffer, frame| {
                    match frame {
                        Frame::Start => { buffer.clear(); Some(None) },
                        Frame::End => Some(Some(buffer.clone())),
                        Frame::DataSet(string) => {
                            match string.split_ascii_whitespace().collect::<Vec<_>>().as_slice() {
                                [key, value, ..] if KEYS.contains(key) => {
                                    let (key, value) = (*key, *value);
                                    let value = match key {
                                        "PTEC" => &value[..2],
                                        _ => value
                                    };
                                    buffer.insert(key.into(), value.into());
                                    Some(None)
                                },
                                _ => Some(None)
                            }
                        },
                    }
                })
                .flatten()
                .filter(|map| KEYS.iter().all(|key| map.contains_key(&key.to_string())))
                .filter_map(move |map| {
                    LinkyFrame::parse(&map, &*dt_gen.clone()).ok()
                        .or_else(|| { log::warn!("couldn't extract LinkyFrame from map: {map:?}"); None })
                });

        Ok(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADCO: &str = "\nADCO 041876097767 U\r";
    const FRAME: &str = "\u{02}\
                         \nADCO 041876097767 U\r\
                         \nOPTARIF HC.. <\r\
                         \nISOUSC 30 9\r\
                         \nHCHC 019650909 -\r\
                         \nHCHP 043280553 1\r\
                         \nPTEC HP..\r\
                         \nIINST1 018 Q\r\
                         \nIINST2 019 S\r\
                         \nIINST3 017 R\r\
                         \nIMAX1 060 6\r\
                         \nIMAX2 060 7\r\
                         \nIMAX3 060 8\r\
                         \nPMAX 10737 8\r\
                         \nPAPP 12690 3\r\
                         \nHHPHC A ,\r\
                         \nMOTDETAT 000000 B\r\
                         \nPPOT 00 #\r
                         \u{03}";

    fn frame(now: DateTime<Utc>) -> LinkyFrame {
        LinkyFrame {
            adco: "041876097767".to_string(),
            ptec: "HP".to_string(),
            hchc: 19_650_909,
            hchp: 43_280_553,
            timestamp: now
        }
    }

    #[test]
    fn test_linky_iterator() {
        // Given
        let now = Utc::now();
        let input = FRAME.chars();
        // When
        let frames = Linky::builder()
            .with_source_iter(input)
            .with_dt_gen(move || now)
            .build().unwrap();
        // Then
        assert_eq!(frames.collect::<Vec<_>>(), vec![frame(now)]);
    }

    #[test]
    fn test_linky_iterator_two_frames() {
        // Given
        let now = Utc::now();
        let input = FRAME.chars().cycle().take(FRAME.len() * 2);
        // When
        let frames = Linky::builder()
            .with_source_iter(input)
            .with_dt_gen(move || now)
            .build().unwrap();
        // Then
        assert_eq!(frames.collect::<Vec<_>>(), vec![frame(now), frame(now)]);
    }

    #[test]
    fn test_linky_iterator_frame_truncated_left() {
        // Given
        let now = Utc::now();
        let input = FRAME[10..].chars().chain(ADCO.chars());
        // When
        let frames = Linky::builder()
            .with_source_iter(input)
            .with_dt_gen(move || now)
            .build().unwrap();
        // Then
        assert_eq!(frames.collect::<Vec<_>>(), Vec::new());
    }

    #[test]
    fn test_linky_iterator_frame_truncated_right() {
        // Given
        let now = Utc::now();
        let input = FRAME[..10].chars().chain(ADCO.chars());
        // When
        let frames = Linky::builder()
            .with_source_iter(input)
            .with_dt_gen(move || now)
            .build().unwrap();
        // Then
        assert_eq!(frames.collect::<Vec<_>>(), Vec::new());
    }

    #[test]
    fn test_linkyframe_parse() {
        // Given
        let now = Utc::now();
        let mut map = HashMap::<String, String>::new();
        map.insert("ADCO".to_string(), "041876097767".to_string());
        map.insert("PTEC".to_string(), "HP".to_string());
        map.insert("HCHC".to_string(), "019650909".to_string());
        map.insert("HCHP".to_string(), "043280553".to_string());
        // When
        let result = LinkyFrame::parse(&map, &|| now).unwrap();
        // Then
        assert_eq!(result, frame(now));
    }

    #[test]
    fn test_linkyframe_parse_with_missing_key() {
        // Given
        let now = Utc::now();
        let map = HashMap::<String, String>::new();
        // When
        let result = LinkyFrame::parse(&map, &|| now);
        // Then
        assert_eq!(result, Err(ParseError));
    }

    #[test]
    fn test_linkyframe_parse_with_extra_keys() {
        // Given
        let now = Utc::now();
        let mut map = HashMap::<String, String>::new();
        map.insert("ADCO".to_string(), "041876097767".to_string());
        map.insert("PTEC".to_string(), "HP".to_string());
        map.insert("HCHC".to_string(), "019650909".to_string());
        map.insert("HCHP".to_string(), "043280553".to_string());
        map.insert("MORE".to_string(), "dummy".to_string());
        // When
        let result = LinkyFrame::parse(&map, &|| now).unwrap();
        // Then
        assert_eq!(result, frame(now));
    }

    #[test]
    fn test_linkyframe_ptec() {
        // Given
        let frame = frame(Utc::now());
        // When & Then
        assert_eq!(TariffPeriod::HC, LinkyFrame { ptec: String::from("HC"), ..frame.clone() }.ptec());
        assert_eq!(TariffPeriod::HP, LinkyFrame { ptec: String::from("HP"), ..frame.clone() }.ptec());
        assert_eq!(TariffPeriod::Unknown, LinkyFrame { ptec: String::from("?"), ..frame.clone() }.ptec());
    }

}
