use actix_web::{web, App, HttpResponse, HttpServer};
use log::{error, info};
use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct AppState {
    markdown: Arc<Mutex<String>>,
}

async fn index(state: web::Data<AppState>) -> HttpResponse {
    let markdown = state.markdown.lock().unwrap().clone();
    let html = markdown_to_html(&markdown);
    let template = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <title>Markdown Editor</title>
            <script>
                function markdownToHtml(markdown) {{
                    return fetch("/render", {{
                        method: "POST",
                        body: markdown
                    }}).then(response => response.text());
                }}
                setInterval(() => {{
                    fetch("/markdown").then(response => {{
                        if (response.ok) {{
                            response.text().then(markdown => {{
                                markdownToHtml(markdown).then(html => {{
                                    document.getElementById("preview").innerHTML = html;
                                }});
                            }});
                        }}
                    }});
                }}, 1000);
            </script>
        </head>
        <body>
            <div id="preview">{}</div>
        </body>
        </html>
    "#,
        html
    );

    return HttpResponse::Ok().content_type("text/html").body(template);
}

async fn update(state: web::Data<AppState>, markdown: web::Bytes) -> HttpResponse {
    if let Err(err) = fs::write("document.md", &markdown) {
        error!("Error writing to file: {}", err);
        return HttpResponse::InternalServerError().body(format!("Error writing to file: {}", err));
    }

    let mut locked_markdown = state.markdown.lock().unwrap();
    match std::str::from_utf8(&markdown) {
        Ok(utf8_string) => {
            *locked_markdown = utf8_string.to_string();
            HttpResponse::Ok().finish()
        }
        Err(err) => {
            error!("Error converting bytes to string: {}", err);
            HttpResponse::InternalServerError()
                .body(format!("Error converting bytes to string: {}", err))
        }
    }
}

async fn render(markdown: web::Bytes) -> HttpResponse {
    let html = markdown_to_html(&String::from_utf8_lossy(&markdown).into_owned());

    return HttpResponse::Ok().body(html);
}

async fn get_markdown() -> HttpResponse {
    let file_name = std::env::args().nth(1).unwrap_or("document.md".to_string());

    match fs::read_to_string(file_name) {
        Ok(markdown) => HttpResponse::Ok().body(markdown),
        Err(err) => {
            error!("Error reading markdown file: {}", err);
            HttpResponse::InternalServerError()
                .body(format!("Error reading markdown file: {}", err))
        }
    }
}

fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();

    html::push_html(&mut html_output, parser);

    return html_output;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let addr = "127.0.0.1:8080";

    let args: Vec<String> = std::env::args().collect();
    let file_name = args.get(1).expect("You did not provide a markdown file.");

    let initial_markdown = match fs::read_to_string(file_name) {
        Ok(markdown) => markdown,
        Err(err) => {
            error!("Error reading initial markdown file: {}", err);
            "".to_string()
        }
    };

    let state = AppState {
        markdown: Arc::new(Mutex::new(initial_markdown)),
    };

    let server = HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .service(web::resource("/").to(index))
            .service(web::resource("/markdown").to(get_markdown))
            .service(web::resource("/update").to(update))
            .service(web::resource("/render").to(render))
            .default_service(web::route().to(index))
    });

    info!("Server started at {}", addr);

    let url = format!("http://{}", addr);
    webbrowser::open(&url).expect("Failed to open browser.");

    server.bind(addr)?.run().await?;

    return Ok(());
}
