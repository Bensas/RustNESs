#[derive(Copy, Debug)]
pub struct Color {
  pub red: u8,
  pub green: u8,
  pub blue: u8
}

impl Color {
  pub fn new(red: u8, green: u8, blue: u8) -> Color {
    return Color { red, green, blue };
  }

  pub fn to_iced_color(&self) -> iced::Color {
    return iced::Color::new((self.red as f32) / 255.0, (self.green as f32) / 255.0, (self.blue as f32) / 255.0, 1.0);
  }
}

impl Clone for Color {
  fn clone(&self) -> Self {
      Self { red: self.red.clone(), green: self.green.clone(), blue: self.blue.clone() }
  }
}