use embedded_graphics::image::ImageRaw;
use embedded_graphics::pixelcolor::BinaryColor;

lazy_static! {
    pub static ref RPICT_OFF: ImageRaw<'static, BinaryColor> = ImageRaw::<BinaryColor>::new(&[
        0b0_0_0_0_0_0_0_0,
        0b0_0_0_1_1_0_0_0,
        0b0_0_1_1_1_1_0_0,
        0b0_0_1_1_1_1_0_0,
        0b0_1_1_1_1_1_1_0,
        0b0_0_1_0_0_1_0_0,
        0b0_0_1_0_0_1_0_0,
        0b0_0_0_0_0_0_0_0,
    ], 8);

    pub static ref RPICT_ON: ImageRaw<'static, BinaryColor> = ImageRaw::<BinaryColor>::new(&[
        0b1_1_1_1_1_1_1_1,
        0b1_1_1_0_0_1_1_1,
        0b1_1_0_0_0_0_1_1,
        0b1_1_0_0_0_0_1_1,
        0b1_0_0_0_0_0_0_1,
        0b1_1_0_1_1_0_1_1,
        0b1_1_0_1_1_0_1_1,
        0b1_1_1_1_1_1_1_1,
    ], 8);

    pub static ref LINKY_OFF: ImageRaw<'static, BinaryColor> = ImageRaw::<BinaryColor>::new(&[
        0b0_0_0_0_0_0_0_0,
        0b0_0_0_0_1_1_0_0,
        0b0_0_0_1_1_0_0_0,
        0b0_0_1_1_0_0_0_0,
        0b0_0_0_0_1_1_0_0,
        0b0_0_0_1_1_0_0_0,
        0b0_0_1_1_0_0_0_0,
        0b0_0_0_0_0_0_0_0,
    ], 8);

    pub static ref LINKY_ON: ImageRaw<'static, BinaryColor> = ImageRaw::<BinaryColor>::new(&[
        0b1_1_1_1_1_1_1_1,
        0b1_1_1_1_0_0_1_1,
        0b1_1_1_0_0_1_1_1,
        0b1_1_0_0_1_1_1_1,
        0b1_1_1_1_0_0_1_1,
        0b1_1_1_0_0_1_1_1,
        0b1_1_0_0_1_1_1_1,
        0b1_1_1_1_1_1_1_1,
    ], 8);

    pub static ref WIFI_OFF: ImageRaw<'static, BinaryColor> = ImageRaw::<BinaryColor>::new(&[
        0b0_0_0_0_0_0_0_0,
        0b0_0_1_1_1_1_0_0,
        0b0_1_0_0_0_0_1_0,
        0b0_0_0_1_1_0_0_0,
        0b0_0_1_0_0_1_0_0,
        0b0_0_0_0_0_0_0_0,
        0b0_0_0_1_1_0_0_0,
        0b0_0_0_0_0_0_0_0,
    ], 8);

    pub static ref WIFI_ON: ImageRaw<'static, BinaryColor> = ImageRaw::<BinaryColor>::new(&[
        0b1_1_1_1_1_1_1_1,
        0b1_1_0_0_0_0_1_1,
        0b1_0_1_1_1_1_0_1,
        0b1_1_1_0_0_1_1_1,
        0b1_1_0_1_1_0_1_1,
        0b1_1_1_1_1_1_1_1,
        0b1_1_1_0_0_1_1_1,
        0b1_1_1_1_1_1_1_1,
    ], 8);
}
