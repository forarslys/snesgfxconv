pub mod image;
pub mod snes;

fn main() {
	let matches = clap::App::new(clap::crate_name!())
		.version(clap::crate_version!())
		.author("forarslys")
		.about("PNG to SNES graphics convertor")
		.arg(clap::Arg::with_name("input").help("input PNG file").required(true))
		.arg(clap::Arg::with_name("2bpp").help("Converts to 2bpp").long("2bpp").short("2"))
		.arg(clap::Arg::with_name("4bpp").help("Converts to 4bpp").long("4bpp").short("4"))
		.arg(clap::Arg::with_name("8bpp").help("Converts to 8bpp").long("8bpp").short("8"))
		.group(clap::ArgGroup::with_name("bpp").args(&["2bpp", "4bpp", "8bpp"]))
		.get_matches();

	let input = matches.value_of("input").unwrap();
	let image = image::Image::open_png(&input).expect("Could not read a PNG file.");
}
