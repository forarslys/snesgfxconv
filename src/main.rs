pub mod image;

use std::io::Write;

fn main() {
	let matches = clap::App::new(clap::crate_name!())
		.version(clap::crate_version!())
		.author("forarslys")
		.about("PNG to SNES graphics convertor")
		.arg(
			clap::Arg::with_name("input")
				.help("input PNG file")
				.required(true),
		)
		.arg(
			clap::Arg::with_name("output")
				.help("output binary file")
				.long("output")
				.short("o")
				.takes_value(true),
		)
		.arg(
			clap::Arg::with_name("palette")
				.help("output palette file")
				.long("palette")
				.short("p")
				.takes_value(true),
		)
		.arg(
			clap::Arg::with_name("tilemap")
				.help("output tilemap file")
				.long("tilemap")
				.short("t")
				.takes_value(true),
		)
		.arg(
			clap::Arg::with_name("dedup")
				.help("Removes duplicate tiles")
				.long("dedup"),
		)
		.arg(
			clap::Arg::with_name("2bpp")
				.help("Converts to 2bpp")
				.long("2bpp")
				.short("2"),
		)
		.arg(
			clap::Arg::with_name("4bpp")
				.help("Converts to 4bpp")
				.long("4bpp")
				.short("4"),
		)
		.arg(
			clap::Arg::with_name("8bpp")
				.help("Converts to 8bpp")
				.long("8bpp")
				.short("8"),
		)
		.group(clap::ArgGroup::with_name("bpp").args(&["2bpp", "4bpp", "8bpp"]))
		.get_matches();

	let input = matches.value_of("input").unwrap();
	let output = matches.value_of("output");
	let tilemap_path = matches.value_of("tilemap");
	let bpp = {
		let mut bpp = None;
		use image::BitsPerPixel::*;
		const FLAG: [&str; 3] = ["2bpp", "4bpp", "8bpp"];
		const BPP: [image::BitsPerPixel; 3] = [Two, Four, Eight];
		for (f, &b) in FLAG.iter().zip(BPP.iter()) {
			if matches.is_present(f) {
				bpp = Some(b);
				break;
			}
		}
		bpp
	};
	let dedup = matches.is_present("dedup");

	let image = match image::Image::open_png(&input) {
		Ok(image) => image,
		Err(msg) => {
			eprintln!("{}", msg);
			std::process::exit(1);
		}
	};

	if let Some(palette) = matches.value_of("palette") {
		let mut file = std::fs::File::create(&palette).unwrap();
		let plte = image.get_palettes();
		let bin = bincode::serialize(plte).unwrap();
		assert_eq!(
			bin[0..8].iter().rev().fold(0u64, |r, &i| r << 8 | i as u64) as usize
				* std::mem::size_of::<sneslib::graphics::color::SNESColor>(),
			bin[8..].len()
		);
		file.write_all(&bin[8..]).unwrap();
	}

	let encoded = if output.is_some() || tilemap_path.is_some() {
		match image.convert_to(bpp, dedup, tilemap_path.is_some()) {
			Ok(enc) => Some(enc),
			Err(msg) => {
				eprintln!("{}", msg);
				eprintln!("Failed in encoding the image.");
				std::process::exit(1);
			}
		}
	} else {
		None
	};

	if let Some(output) = output {
		if let Some((bin, _)) = encoded.as_ref() {
			let mut file = std::fs::File::create(&output).unwrap();
			file.write_all(bin.as_slice()).unwrap();
		}
	}

	if let Some(tilemap_path) = tilemap_path {
		match encoded.as_ref() {
			Some((_, Some(Ok(tilemap)))) => {
				let mut file = std::fs::File::create(&tilemap_path).unwrap();
				file.write_all(tilemap.as_slice()).unwrap();
			}
			Some((_, Some(Err(msg)))) => {
				eprintln!("{}", msg);
				eprintln!("Failed in exporting a tilemap file.");
				std::process::exit(1);
			}
			_ => (),
		}
	}
}
