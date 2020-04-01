use super::snes::gfx;

pub struct Image {
	buffer: Vec<u8>,
	plte: Vec<gfx::SNESColor>,
	width: u32,
	height: u32,
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
		Ok(Self { buffer, plte, width: info.width, height: info.height })
	}
}
