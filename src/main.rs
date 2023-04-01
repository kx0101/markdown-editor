use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use pulldown_cmark::{html, Options, Parser};
use std::fs::{self, File};
use std::io::{Result, Write};
use std::path::Path;
use std::sync::mpsc::channel;
use tide::{Body, Request};

fn read_md_file(file_path: &Path) -> Result<String> {
    let md = fs::read_to_string(file_path.to_str().unwrap())?;

    return Ok(md);
}

//  <meta http-equiv=\"refresh\" content=\"1\">\n\
fn write_html_file(file_path: &Path, html: &str) -> Result<()> {
    let mut file = File::create(file_path)?;
    let html_with_meta = format!(
        "<!DOCTYPE html>\n<html>\n<head>\n\
        <meta http-equiv=\"Cache-Control\" content=\"no-cache, no-store, must-revalidate\">\n\
        <meta http-equiv=\"refresh\" content=\"1\">\n\
        <script src=\"https://cdnjs.cloudflare.com/ajax/libs/socket.io/3.1.3/socket.io.js\"></script>\n\
        <script src=\"https://cdnjs.cloudflare.com/ajax/libs/jquery/3.6.0/jquery.min.js\"></script>\n\
        <script defer>\n\
            const socket = io.connect('http://localhost:4000/socket.io');\n\
            socket.on('reload', function() {{ location.reload(); }});\n\
        </script>\n\
        </head>\n\
        <body>\n{}\n</body>\n</html>",
        html
    );

    write!(file, "{}", html_with_meta)?;
    file.sync_all()?;

    return Ok(());
}

fn markdown_to_html(input_path: &Path, output_path: &Path) {
    let markdown_input = read_md_file(input_path).unwrap();

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&markdown_input, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    write_html_file(output_path, &html_output).unwrap();
}

#[tokio::main]
async fn main() -> tide::Result<()> {
    env_logger::init();
    std::env::set_var("RUST_BACKTRACE", "1");
    let mut app = tide::new();

    let input_file_path = Path::new("input.md");
    let output_file_path = Path::new("output.html");

    app.at("/")
        .get(|_| async { Ok(Body::from_file("output.html").await?) });

    app.at("/socket.io/").get(|_req: Request<()>| async move {
        let mut res = tide::http::Response::new(200);

        res.insert_header("Connection", "keep-alive");
        res.set_body("reload\n");

        Ok(res)
    });

    let (tx, rx) = channel();

    tokio::task::spawn_blocking(move || {
        let mut input_watcher: RecommendedWatcher =
            Watcher::new(tx.clone(), notify::Config::default()).unwrap();
        input_watcher
            .watch(input_file_path, RecursiveMode::NonRecursive)
            .unwrap_or_else(|err| {
                panic!("Failed to watch input file: {:?}", err);
            });

        let mut output_watcher: RecommendedWatcher =
            Watcher::new(tx, notify::Config::default()).unwrap();
        output_watcher
            .watch(output_file_path, RecursiveMode::NonRecursive)
            .unwrap_or_else(|err| {
                panic!("Failed to watch output file: {:?}", err);
            });

        loop {
            match rx.recv() {
                Ok(_) => markdown_to_html(input_file_path, output_file_path),
                Err(err) => {
                    println!("watch error: {:?}", err);
                    break;
                }
            };
        }
    });

    app.listen("localhost:4000").await?;

    return Ok(());
}
