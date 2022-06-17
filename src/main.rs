use clap::Parser;

/// Download doujinshi from nhentai.net
#[derive(Parser)]
struct Args {
	/// Code used to find doujinshi
	#[clap(short, long, value_parser)]
	code: u32,
}

fn main() {
	let args = Args::parse();

	let pages = fetch_pages(args.code);

	if pages.len() > 0 {
		println!("Found {} pages", pages.len());
		std::fs::create_dir(format!("{}", args.code)).unwrap();
	} else {
		println!("Invalid nhentai code")
	}

	for (i, url) in pages.iter().enumerate() {
		print!("Page {} ", i + 1);
		download_image(fetch_image_url(url.to_string()), format!("{}/{i}.png", args.code));
	}
}

fn fetch_pages(code: u32) -> Vec<String> {
	let url = format!("https://nhentai.net/g/{}", code);
	let response = reqwest::blocking::get(url).unwrap().text().unwrap();
	let document = scraper::Html::parse_document(&response);
	let page_selector = scraper::Selector::parse("div.thumb-container > a").unwrap();
	let pages: Vec<String> = document.select(&page_selector).map(|elem| format!("https://nhentai.net{}", elem.value().attr("href").unwrap())).collect();
	return pages
}

fn fetch_image_url(url: String) -> String {
	let response = reqwest::blocking::get(url).unwrap().text().unwrap();
	let document = scraper::Html::parse_fragment(&response);
	let img_selector = scraper::Selector::parse("section#image-container > a > img").unwrap();
	let img_url = String::from(document.select(&img_selector).next().unwrap().value().attr("src").unwrap());
	return img_url
}

fn download_image(url: String, file_name: String) {
	let img_bytes = reqwest::blocking::get(url).unwrap().bytes().unwrap();
	let img = image::load_from_memory(&img_bytes).unwrap();
	img.save(&file_name).unwrap();
	println!("saved as {file_name}");
}