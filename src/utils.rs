pub mod bitwise_utils {
  pub fn get_bit(source: u8, bit_pos: u8) -> u8{ // bit_pos counted from least significant to most significant
    return (source & (1 << bit_pos) != 0) as u8;
  }

  pub fn get_bit_16(source: u16, bit_pos: u8) -> u8{
    return (source & (1 << bit_pos) != 0) as u8;
  }

  pub fn set_bit(target: &mut u8, bit_pos: u8, new_value: u8) {
    match new_value {
      0 => *target &= !(1 << bit_pos),
      1 => *target |= (1 << bit_pos),
      _ => panic!("Tried to set_bit with a value other than 0 or 1")
    }
  }

  pub fn set_bit_16(target: &mut u16, bit_pos: u8, new_value: u8) {
    match new_value {
      0 => *target &= !(1 << bit_pos),
      1 => *target |= (1 << bit_pos),
      _ => panic!("Tried to set_bit_16 with a value other than 0 or 1")
    }
  }

  pub fn get_bits_16(source: u16, start_bit_pos: u8, end_bit_pos: u8) -> u16 { // start and end are inclusive
    let mut mask: u16 = 0b00000000;
    for i in start_bit_pos..end_bit_pos+1 {
      mask |= 1 << i;
    }
    return (source & mask) >> start_bit_pos;
  }

  pub fn set_bits_16(target: &mut u16, start_bit_pos: u8, end_bit_pos: u8, new_value: u16) {
    for i in start_bit_pos..end_bit_pos+1 {
      set_bit_16(target, i, get_bit_16(new_value, i - start_bit_pos));
    }
  }
}

pub mod hex_utils {
  pub fn decimal_word_to_hex_str(word: u16) -> String {
    let mut result = String::new();

    let mut factor = 16 as u32;
    let mut curr_val = word as u32;
    while (curr_val != 0) {
      let digit = (curr_val % factor) / (factor/16);
      result.push(decimal_value_to_hex_char(digit as u8));
      curr_val -= curr_val % factor;
      factor *= 16;
    }
    
    return result.chars().rev().collect();
  }

  pub fn decimal_byte_to_hex_str(decimal: u8) -> String {
    let mut result = String::new();
    let least_sig_digit_value = decimal % 16;
    let most_sig_digit_value = (decimal - least_sig_digit_value) / 16;
    result.push(decimal_value_to_hex_char(most_sig_digit_value));
    result.push(decimal_value_to_hex_char(least_sig_digit_value));
    return result;
  }

  fn decimal_value_to_hex_char(val: u8) -> char {
    match val {
      10 => return 'A',
      11 => return 'B',
      12 => return 'C',
      13 => return 'D',
      14 => return 'E',
      15 => return 'F',
      _ => return char::from_digit(val.into(), 10).unwrap()
    }
  }
}