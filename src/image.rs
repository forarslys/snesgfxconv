use super::snes::gfx;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BitsPerPixel {
	Two,
	Four,
	Eight,
}

pub struct Image {
	buffer: Vec<u8>,
	plte: Vec<gfx::SNESColor>,
	width: u32,
	height: u32,
	bpp: BitsPerPixel,
}

impl Image {
	pub fn open_png<P: AsRef<std::path::Path>>(path: P) -> Result<Self, &'static str> {
		let file = std::fs::File::open(path).expect("Could not open a file.");
		let mut decoder = png::Decoder::new(file);
		decoder.set_transformations(png::Transformations::IDENTITY | png::Transformations::PACKING);
		let (info, mut reader) = decoder.read_info().expect("Could not decode the file.");

		if info.color_type != png::ColorType::Indexed {
			return Err("Color type of the PNG file must be Indexed Color.");
		}

		if info.width % 8 != 0 || info.height % 8 != 0 {
			return Err("Width and height of the image must be multiples of 8.");
		}

		let mut buffer = vec![0; info.buffer_size()];
		reader.next_frame(&mut buffer).unwrap();

		let image_info = reader.info();
		let mut plte = vec![];
		let palette = image_info.palette.as_ref().unwrap();
		for i in (0..palette.len()).step_by(3) {
			plte.push(gfx::Color(palette[i], palette[i] + 1, palette[i] + 2).into());
		}

		let bpp = Self::analyze_bpp(&buffer, info.width as usize, info.height as usize);

		Ok(Self { buffer, plte, width: info.width, height: info.height, bpp })
	}

	fn analyze_bpp(buffer: &[u8], width: usize, height: usize) -> BitsPerPixel {
		let mut min_bpp = BitsPerPixel::Two;
		for y in (0..height).step_by(8) {
			for x in (0..width).step_by(8) {
				let mut min_index = 255;
				let mut max_index = 0;
				for ix in 0..8 {
					for iy in 0..8 {
						let offset = (x + ix) + (y + iy) * width;
						if buffer[offset] > max_index {
							max_index = buffer[offset];
						}
						if buffer[offset] < min_index {
							min_index = buffer[offset];
						}
					}
				}
				let bpp = if (max_index & !3) == (min_index & !3) {
					BitsPerPixel::Two
				} else if (max_index & !15) == (min_index & !15) {
					BitsPerPixel::Four
				} else {
					BitsPerPixel::Eight
				};
				if min_bpp < bpp {
					min_bpp = bpp;
				}
			}
		}
		min_bpp
	}
}
