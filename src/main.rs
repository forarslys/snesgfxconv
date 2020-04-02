pub mod image;
pub mod snes;

fn main() {
	let matches = clap::App::new(clap::crate_name!())
		.version(clap::crate_version!())
		.author("forarslys")
		.about("PNG to SNES graphics convertor")
		.arg(clap::Arg::with_name("input").help("input PNG file").required(true))
		.arg(clap::Arg::with_name("output").help("output binary file").long("output").short("o").takes_value(true))
		.arg(clap::Arg::with_name("palette").help("output palette file").long("palette").short("p").takes_value(true))
		.arg(clap::Arg::with_name("dedup").help("Removes duplicate tiles").long("dedup"))
		.arg(clap::Arg::with_name("2bpp").help("Converts to 2bpp").long("2bpp").short("2"))
		.arg(clap::Arg::with_name("4bpp").help("Converts to 4bpp").long("4bpp").short("4"))
		.arg(clap::Arg::with_name("8bpp").help("Converts to 8bpp").long("8bpp").short("8"))
		.group(clap::ArgGroup::with_name("bpp").args(&["2bpp", "4bpp", "8bpp"]))
		.get_matches();

	let input = matches.value_of("input").unwrap();
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

	let image = image::Image::open_png(&input).expect("Could not read a PNG file.");
	if let Some(output) = matches.value_of("output") {
		if let Ok(bin) = image.convert_to(bpp, dedup) {
			let mut file = std::fs::File::create(&output).unwrap();
			use std::io::Write;
			file.write_all(bin.as_slice()).unwrap();
		}
	}

	if let Some(palette) = matches.value_of("palette") {
		let mut file = std::fs::File::create(&palette).unwrap();
		use std::io::Write;
		let plte = image.get_palettes();
		unsafe {
			let slice = std::slice::from_raw_parts(plte.as_ptr() as *const u8, plte.len() * std::mem::size_of::<snes::gfx::SNESColor>());
			file.write_all(&slice).unwrap();
		}
	}
}
