use embedded_graphics::primitives::StyledDrawable;
use embedded_graphics::{
    mono_font::{ascii::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};

#[derive(Clone, Debug, PartialEq)]
pub struct SparkLine {
    bottom_left: Point,
    label: String,
    max_value: f32,
    value: f32,
    value_max: f32,
}

impl SparkLine {
    pub fn new(bottom_left: Point, label: String, max_value: f32) -> Self {
        Self {
            bottom_left,
            label: label[..2].to_string(),
            max_value,
            value: 0.0,
            value_max: 0.0,
        }
    }

    pub fn update(&mut self, value: f32) {
        self.value = value;
        if value > self.value_max {
            self.value_max = value;
        }
    }
}

impl Drawable for SparkLine {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let border_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 2);
        let text_style = MonoTextStyle::new(&FONT_5X7, BinaryColor::On);

        // draw heading text
        let point = Text::with_alignment(
            &format!("{} {:4.0}W", self.label, self.value),
            self.bottom_left,
            text_style,
            Alignment::Left,
        )
        .draw(target)?;

        // draw sparkline border
        let border = Rectangle::with_corners(
            point,
            Point::new(target.bounding_box().size.width as i32 - 2, point.y - 5),
        );
        border.into_styled(border_style).draw(target)?;
        let available_width = border.size.width as i32 - 5;

        // draw sparkline value
        let start_point = border.top_left + Point::new(2, 3);
        let value_width = (self.value.clamp(0.0, self.max_value) / self.max_value) * available_width as f32;
        let end_point = start_point + Point::new(value_width as i32, 0);
        Line::new(start_point, end_point).draw_styled(&line_style, target)?;

        // draw sparkline value max
        let value_max_width = (self.value_max.clamp(0.0, self.max_value) / self.max_value) * available_width as f32;
        let max_point = start_point + Point::new(value_max_width as i32, 0);
        Line::new(max_point, max_point).draw_styled(&line_style, target)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparkline_new() {
        // Given
        let point = Point::new(0, 0);
        let string = String::from("label");
        // When
        let actual = SparkLine::new(point, string, 8000.0);
        // Then
        assert!(
            matches!(actual, SparkLine { bottom_left, label, max_value, value, value_max }
            if bottom_left == point
            && label == "la"
            && max_value == 8000.0
            && value == 0.0
            && value_max == 0.0)
        );
    }

    #[test]
    fn test_sparkline_update() {
        // Given
        let point = Point::new(0, 0);
        let string = String::from("label");
        let mut actual = SparkLine::new(point, string, 8000.0);
        // Case 1
        actual.update(100.0);
        assert!(matches!(actual, SparkLine { value, value_max, .. } if value == 100.0 && value_max == 100.0));
        // Case 2
        actual.update(200.0);
        assert!(matches!(actual, SparkLine { value, value_max, .. } if value == 200.0 && value_max == 200.0));
        // Case 3
        actual.update(50.0);
        assert!(matches!(actual, SparkLine { value, value_max, .. } if value == 50.0 && value_max == 200.0));
    }
}
