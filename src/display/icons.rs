use embedded_graphics::image::ImageRaw;
use embedded_graphics::pixelcolor::BinaryColor;

lazy_static! {
    pub static ref RPICT_OFF: ImageRaw<'static, BinaryColor> = ImageRaw::<BinaryColor>::new(&[
        0b0000_0000,
        0b0001_1000,
        0b0011_1100,
        0b0011_1100,
        0b0111_1110,
        0b0010_0100,
        0b0010_0100,
        0b0000_0000,
    ], 8);

    pub static ref RPICT_ON: ImageRaw<'static, BinaryColor> = ImageRaw::<BinaryColor>::new(&[
        0b1111_1111,
        0b1110_0111,
        0b1100_0011,
        0b1100_0011,
        0b1000_0001,
        0b1101_1011,
        0b1101_1011,
        0b1111_1111,
    ], 8);

    pub static ref LINKY_OFF: ImageRaw<'static, BinaryColor> = ImageRaw::<BinaryColor>::new(&[
        0b0000_0000,
        0b0000_1100,
        0b0001_1000,
        0b0011_0000,
        0b0000_1100,
        0b0001_1000,
        0b0011_0000,
        0b0000_0000,
    ], 8);

    pub static ref LINKY_ON: ImageRaw<'static, BinaryColor> = ImageRaw::<BinaryColor>::new(&[
        0b1111_1111,
        0b1111_0011,
        0b1110_0111,
        0b1100_1111,
        0b1111_0011,
        0b1110_0111,
        0b1100_1111,
        0b1111_1111,
    ], 8);

    pub static ref WIFI_OFF: ImageRaw<'static, BinaryColor> = ImageRaw::<BinaryColor>::new(&[
        0b0000_0000,
        0b0011_1100,
        0b0100_0010,
        0b0001_1000,
        0b0010_0100,
        0b0000_0000,
        0b0001_1000,
        0b0000_0000,
    ], 8);

    pub static ref WIFI_ON: ImageRaw<'static, BinaryColor> = ImageRaw::<BinaryColor>::new(&[
        0b1111_1111,
        0b1100_0011,
        0b1011_1101,
        0b1110_0111,
        0b1101_1011,
        0b1111_1111,
        0b1110_0111,
        0b1111_1111,
    ], 8);
}
