use rayon::prelude::*;

const BASE_URL: &str = "https://nhentai.to";

#[tokio::main]
async fn main() {
	// Get code from CLI args
	let args: Vec<String> = std::env::args().collect();
	let code = &args[1];

	// Fetch base page
	let url = format!("{BASE_URL}/g/{code}/");
	let res = reqwest::get(url)
		.await.unwrap()
		.text()
		.await.unwrap();

	// Parse base page
	let doc = scraper::Html::parse_fragment(&res);
	let page_selector = scraper::Selector::parse("a.gallerythumb").unwrap();

	// Grab urls of pages
	let page_urls: Vec<String> = doc.select(&page_selector).map(|elem| {
		format!("{BASE_URL}/{}", elem.value().attr("href").unwrap())
	}).collect();

	// Check if the code is valid by checking if page_urls has at least 1 page url
	if !page_urls.is_empty() {
		println!("Found {} pages from {}", page_urls.len(), code);
		std::fs::create_dir_all(code).unwrap(); // Create destination directory for images
	} else {
		print!("Code returned 0 pages");
	}

	// Iterate over page urls using rayon to get the urls of the actual images we want to download
	let img_urls: Vec<String> = page_urls.par_iter().map(|url| {
		let res = reqwest::blocking::get(url)
			.unwrap()
			.text()
			.unwrap();

		// Fetch image page
		let doc = scraper::Html::parse_fragment(&res);
		let img_selector = scraper::Selector::parse("section#image-container > a > img").unwrap();

		// Grab src-attribute from images
		let img_url = doc.select(&img_selector).next().unwrap().value().attr("src").unwrap();

		img_url.to_string()
	}).collect();

	// Iterate over image urls using rayon and save the images as PNGs
	img_urls.par_iter().enumerate().map(|(i, url)| {
		let dst = format!("{code}/{i}.png");

		let img_bytes = reqwest::blocking::get(url.to_string())
			.unwrap()
			.bytes()
			.unwrap();

		image::load_from_memory(&img_bytes).unwrap().save(&dst).unwrap();

		println!("{:0>2?} - Saved page {} as {dst}", std::thread::current().id(), i + 1);
	}).collect::<()>();

	println!("Finished downloading {} images", img_urls.len());
}
