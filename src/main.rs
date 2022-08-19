use rayon::prelude::*;

const BASE_URL: &str = "https://nhentai.to";

#[tokio::main]
async fn main() {
	// Get code from CLI args
	// If no code was passed as an argument default to 0
	let args: Vec<String> = std::env::args().collect();
	let code = if args.len() <= 1 {"0"} else {&args[1]};

	// Fetch base page
	let url = format!("{BASE_URL}/g/{code}/");
	let res = reqwest::get(url)
		.await
		.unwrap()
		.text()
		.await
		.unwrap();

	// Parse base page
	let doc = scraper::Html::parse_fragment(&res);
	let page_selector = scraper::Selector::parse("a.gallerythumb").unwrap();

	// Grab urls of pages
	let page_urls: Vec<String> = doc.select(&page_selector).map(|elem| {
		format!("{BASE_URL}{}", elem.value().attr("href").unwrap())
	}).collect();

	// Check if the code is valid by checking if page_urls has at least 1 page url
	if !page_urls.is_empty() {
		println!("Found {} pages from {code}", page_urls.len());
		std::fs::create_dir_all(code).unwrap(); // Create destination directory for images
	} else {
		println!("Found 0 pages from {code}");
	}

	// Iterate over page urls using rayon to get the urls of the actual images we want to download
	page_urls.par_iter().enumerate().map(|(i, url)| {
		let res = fetch_page(url);

		// Fetch image page
		let doc = scraper::Html::parse_fragment(&res);
		let img_selector = scraper::Selector::parse("section#image-container > a > img").unwrap();

		// Grab src-attribute from images
		let img_url = doc.select(&img_selector)
			.next()
			.unwrap()
			.value()
			.attr("src")
			.unwrap();

		println!("{:0>2?} - Discovered {img_url}", std::thread::current().id());

		// Save images as PNGs
		let dst = format!("{code}/{i}.png");
		let img_bytes = reqwest::blocking::get(img_url)
			.unwrap()
			.bytes()
			.unwrap();
		image::load_from_memory(&img_bytes)
			.unwrap()
			.save(&dst)
			.unwrap();

		println!("{:0>2?} - Saved page {} as {dst}", std::thread::current().id(), i + 1);
	}).collect::<()>();

	println!("Finished downloading {} images", page_urls.len());
}

// Function fetches the page using the url
// If the reqwest times out then we try again
// This could soft lock the program...
fn fetch_page(url: &str) -> String {
	match reqwest::blocking::get(url) {
		Ok(res) => res.text().unwrap(),
		Err(_) => {
			println!("{} - Connection timed out, retrying...", format!("{:0>2?}", std::thread::current().id()).red());
			fetch_page(url)
		},
	}
}
