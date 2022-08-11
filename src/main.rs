use std::{
	io::{stdout, Write},
	sync::mpsc::{self, TryRecvError},
	thread,
	time::Duration,
	process,
};

use clap::Parser;

use colored::Colorize;

use question::{
	Question, Answer
};

use crossterm::{
	cursor,
	ExecutableCommand,
};

/// Download doujinshi from nhentai.net
#[derive(Parser)]
struct Args {
	/// Code used to find doujinshi
	#[clap(short, long, value_parser)]
	code: u32,
}

fn main() {
	stdout().execute(cursor::Hide).unwrap();

	let args = Args::parse();

	let pages = fetch_pages(args.code);

	if !pages.is_empty() {
		println!("Found {} pages from {}", pages.len().to_string().yellow(), args.code.to_string().yellow());
		std::fs::create_dir_all(format!("{}", args.code)).unwrap();
	} else {
		println!("Invalid nhentai code")
	}

	let answer = Question::new(format!("Are you sure you want to download {}?", args.code.to_string().yellow()).as_str())
		.yes_no()
		.until_acceptable()
		.default(Answer::YES)
		.show_defaults()
		.ask();

	/* Handle CTRL+C signal */
	ctrlc::set_handler(move || {
		print!("{} {}", String::from('✗').red(), "Interrupted".bright_red());
		stdout().execute(cursor::Show).unwrap();
		process::exit(0);
	}).unwrap();

	if answer == Some(Answer::YES) {
		download_images(args, pages);
	}

	stdout().execute(cursor::Show).unwrap();
}

fn fetch_pages(code: u32) -> Vec<String> {
	let url = format!("https://nhentai.to/g/{}/", code);
	let response = reqwest::blocking::get(url).unwrap().text().unwrap();
	let document = scraper::Html::parse_document(&response);
	let page_selector = scraper::Selector::parse("a.gallerythumb").unwrap();
	let pages: Vec<String> = document.select(&page_selector).map(|elem| format!("https://nhentai.to{}", elem.value().attr("href").unwrap())).collect();
	pages
}

fn fetch_image_url(url: String) -> String {
	let response = reqwest::blocking::get(url).unwrap().text().unwrap();
	let document = scraper::Html::parse_fragment(&response);
	let img_selector = scraper::Selector::parse("section#image-container > a > img").unwrap();
	let img_url = String::from(document.select(&img_selector).next().unwrap().value().attr("src").unwrap());
	img_url
}

fn download_image(url: String, file_name: String) {
	let img_bytes = reqwest::blocking::get(url).unwrap().bytes().unwrap();
	let img = image::load_from_memory(&img_bytes).unwrap();
	img.save(&file_name).unwrap();
}

fn download_images(args: Args, pages: Vec<String>) {
	let (tx, rx) = mpsc::channel();

	for (i, url) in pages.into_iter().enumerate() {
		let tx = tx.clone();

		thread::spawn(move || {
			download_image(fetch_image_url(url.to_string()), format!("{}/{i}.png", args.code));
			tx.send(()).unwrap();
		});

		let mut counter = 0;

		loop {
			let stage = match counter % 6 {
				0 => '⠏',
				1 => '⠛',
				2 => '⠹',
				3 => '⠼',
				4 => '⠶',
				5 => '⠧',
				_ => ' ',
			};

			print!("{} {} Page {}...\r", String::from(stage).blue(), "Downloading".bright_cyan(), (i + 1).to_string().yellow());

			stdout().flush().unwrap();

			thread::sleep(Duration::from_millis(100));

			counter += 1;

			match rx.try_recv() {
				Ok(_) | Err(TryRecvError::Disconnected) => {break;}
				Err(TryRecvError::Empty) => {}
			}
		}

		println!("{} {} Page {} saved as {}/{i}.png", String::from('✓').green(), "   Finished".bright_green(), (i + 1).to_string().yellow(), args.code);
	}
}
