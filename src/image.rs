use super::snes::gfx;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BitsPerPixel {
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
			plte.push(gfx::Color(palette[i], palette[i + 1], palette[i + 2]).into());
		}

		let bpp = Self::analyze_bpp(&buffer, info.width as usize, info.height as usize);

		Ok(Self {
			buffer,
			plte,
			width: info.width,
			height: info.height,
			bpp,
		})
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

	pub fn convert_to(
		&self,
		bpp: Option<BitsPerPixel>,
		dedup: bool,
		export_tilemap: bool,
	) -> Result<(Vec<u8>, Option<Result<Vec<u8>, &'static str>>), &'static str> {
		let bpp = if let Some(bpp) = bpp {
			if bpp < self.bpp {
				return Err("Invalid bpp specified.");
			}
			bpp
		} else {
			self.bpp
		};

		match bpp {
			BitsPerPixel::Two => Ok(self.convert_to_2bpp(dedup, export_tilemap)),
			BitsPerPixel::Four => Ok(self.convert_to_4bpp(dedup, export_tilemap)),
			BitsPerPixel::Eight => Ok(self.convert_to_8bpp(dedup, export_tilemap)),
		}
	}

	pub fn get_palettes(&self) -> &Vec<gfx::SNESColor> {
		&self.plte
	}
}

macro_rules! declare_convert_to {
	($fn:ident, $bpp:expr, $tile_size:expr) => {
		fn $fn(
			&self,
			dedup: bool,
			export_tilemap: bool,
		) -> (Vec<u8>, Option<Result<Vec<u8>, &'static str>>) {
			const TILE_SIZE: usize = $tile_size;

			let mut r = vec![];
			let mut map = std::collections::HashMap::new();
			let mut tilemap = vec![];

			for y in (0..self.height).step_by(8) {
				for x in (0..self.width).step_by(8) {
					let mut tile = vec![vec![0; TILE_SIZE]; 4];
					let palette = match $bpp {
						BitsPerPixel::Two => self.buffer[(x + y * self.width) as usize] >> 2,
						BitsPerPixel::Four => self.buffer[(x + y * self.width) as usize] >> 4,
						BitsPerPixel::Eight => 0,
					};
					macro_rules! encode_tile {
						($id:expr, $yxor:expr, $xxor:expr) => {
							for iy in 0..8 {
								for ix in 0..8 {
									let offset = ((x + (ix ^ $xxor))
										+ (y + (iy ^ $yxor)) * self.width) as usize;
									macro_rules! write_bit {
										($bit:expr, $offset:expr) => {
											if self.buffer[offset] & $bit != 0 {
												tile[$id][2 * iy as usize + $offset] |= 0x80 >> ix;
												}
										};
										}
									if $bpp >= BitsPerPixel::Two {
										write_bit!(0x01, 0x00);
										write_bit!(0x02, 0x01);
										}
									if $bpp >= BitsPerPixel::Four {
										write_bit!(0x04, 0x10);
										write_bit!(0x08, 0x11);
										}
									if $bpp >= BitsPerPixel::Eight {
										write_bit!(0x10, 0x20);
										write_bit!(0x20, 0x21);
										write_bit!(0x40, 0x30);
										write_bit!(0x80, 0x31);
										}
									}
								}
						};
					}
					encode_tile!(0, 0, 0);
					if dedup {
						encode_tile!(1, 0, 7);
						encode_tile!(2, 7, 0);
						encode_tile!(3, 7, 7);

						let mut exists = false;
						for i in 0..4 {
							if let Some(&index) = map.get(&tile[i]) {
								tilemap.push((index, palette, i as u8));
								exists = true;
								break;
							}
						}
						if !exists {
							let index = r.len() / TILE_SIZE;
							map.insert(tile[0].clone(), index);
							tilemap.push((index, palette, 0));
							r.extend_from_slice(&tile[0]);
						}
					} else {
						tilemap.push((r.len() / TILE_SIZE, palette, 0));
						r.extend_from_slice(&tile[0]);
					}
				}
			}

			let tilemap = if export_tilemap {
				let mut valid = true;
				let mut r = vec![];
				for (index, palette, flip) in tilemap {
					if index >= 0x400 {
						valid = false;
						break;
					}
					r.push((index & 0xFF) as u8);
					r.push((index >> 8) as u8 | palette << 2 | flip << 6);
				}
				if valid {
					Some(Ok(r))
				} else {
					Some(Err("Invalid tilemap"))
				}
			} else {
				None
			};
			(r, tilemap)
		}
	};
}

impl Image {
	declare_convert_to!(convert_to_2bpp, BitsPerPixel::Two, 0x10);
	declare_convert_to!(convert_to_4bpp, BitsPerPixel::Four, 0x20);
	declare_convert_to!(convert_to_8bpp, BitsPerPixel::Eight, 0x40);
}
