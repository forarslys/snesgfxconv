#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color(pub u8, pub u8, pub u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SNESColor(pub u16);

impl From<SNESColor> for Color {
	fn from(color: SNESColor) -> Self {
		let SNESColor(color) = color;
		let r = (color & 0x1F) as u8;
		let g = ((color >> 5) & 0x1F) as u8;
		let b = ((color >> 10) & 0x1F) as u8;
		Color(r << 3, g << 3, b << 3)
	}
}

impl From<Color> for SNESColor {
	fn from(color: Color) -> SNESColor {
		let Color(r, g, b) = color;
		let r = std::cmp::min(r as u16 + 4, 0xF8) & 0xF8;
		let g = std::cmp::min(g as u16 + 4, 0xF8) & 0xF8;
		let b = std::cmp::min(b as u16 + 4, 0xF8) & 0xF8;
		SNESColor(r >> 3 | g << 2 | b << 7)
	}
}

#[test]
fn test() {
	assert_eq!(SNESColor(0x7fff), Color(0xf8, 0xf8, 0xf8).into());
	assert_eq!(SNESColor(0x4210), Color(0x80, 0x80, 0x80).into());
	assert_eq!(SNESColor(0x001f), Color(0xf8, 0x00, 0x00).into());
	assert_eq!(SNESColor(0x03e0), Color(0x00, 0xf8, 0x00).into());
	assert_eq!(SNESColor(0x7c00), Color(0x00, 0x00, 0xf8).into());
	assert_eq!(Color(0xf8, 0xf8, 0xf8), SNESColor(0x7fff).into());
	assert_eq!(Color(0x80, 0x80, 0x80), SNESColor(0x4210).into());
	assert_eq!(Color(0xf8, 0x00, 0x00), SNESColor(0x001f).into());
	assert_eq!(Color(0x00, 0xf8, 0x00), SNESColor(0x03e0).into());
	assert_eq!(Color(0x00, 0x00, 0xf8), SNESColor(0x7c00).into());
}
