use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use pulldown_cmark::{html, Options, Parser};
use std::fs::{self, File};
use std::io::{Result, Write};
use std::path::Path;
use std::sync::mpsc::channel;
use async_tungstenite::tungstenite::WebSocket;

async fn hello(_req: tide::Request<()>) -> tide::Result<String> {
    return Ok("Hello, world!".to_string());
}

fn read_md_file(file_path: &Path) -> Result<String> {
    let md = fs::read_to_string(file_path.to_str().unwrap())?;

    return Ok(md);
}

fn write_html_file(file_path: &Path, html: &str) -> Result<()> {
    let mut file = File::create(file_path)?;
    let html_with_meta = format!(
        "<!DOCTYPE html>\n<html>\n<head>\n\
        <meta http-equiv=\"Cache-Control\" content=\"no-cache, no-store, must-revalidate\">\n\
        <script src=\"https://cdnjs.cloudflare.com/ajax/libs/socket.io/3.1.3/socket.io.js\"></script>\n\
        <script src=\"https://cdnjs.cloudflare.com/ajax/libs/jquery/3.6.0/jquery.min.js\"></script>\n\
        <script defer>\n\
            const socket = io.connect('http://localhost:8080');\n\
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

fn markdown_to_html(input_path: &Path, output_path: &Path, ws: &mut WebSocket) {
    let markdown_input = read_md_file(input_path).unwrap();

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&markdown_input, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    write_html_file(output_path, &html_output).unwrap();

    let _ = ws.write_message(tungstenite::Message::Text("reload".to_owned()));
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();

    let input_file_path = Path::new("input.md");
    let output_file_path = Path::new("output.html");

    app.at("/").get(hello);

    let mut ws = app.at("/ws").get(tide::with_upgrade(async {
        |request: tide::Request<()>| {
            let ws = request
                .upgrade()
                .map(|upgraded| {
                    WebSocket::from_raw_socket(
                        upgraded,
                        async_tungstenite::tungstenite::protocol::Role::Server,
                        None,
                    )
                })
                .unwrap();

            async move { while let Ok(_) = ws.read_message().await {} }
        }
    }));

    let (tx, rx) = channel();
    let mut input_watcher: RecommendedWatcher =
        Watcher::new(tx.clone(), notify::Config::default()).unwrap();
    input_watcher.watch(input_file_path, RecursiveMode::NonRecursive)?;

    let mut output_watcher: RecommendedWatcher =
        Watcher::new(tx, notify::Config::default()).unwrap();
    output_watcher.watch(output_file_path, RecursiveMode::NonRecursive)?;

    app.listen("localhost:8080").await?;

    loop {
        match rx.recv() {
            Ok(_) => {
                markdown_to_html(input_file_path, output_file_path, ws);
            }
            Err(err) => println!("watch error: {:?}", err),
        };
    }
}
