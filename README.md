# Markdown Editor

Markdown Editor is a simple web-based editor built with Rust, Actix-Web, and pulldown_cmark that allows you to edit Markdown files on your local machine and see a live preview of the rendered HTML in your web browser.

## Prerequisites
1. Install Rust on your system.

## How to Use

1. Clone this repository to your local machine.
2. Open your preferred IDE or text editor and navigate to the cloned repository.
3. Open the Markdown file that you want to edit.
4. In your terminal, navigate to the root directory of the cloned repository and run the command `cargo run <file.md>` where `<file.md>` is the name of the Markdown file you want to edit.

## What's happening exactly

The program will start a local web server and open your web browser to the editor page.
As you edit the Markdown file in your IDE or text editor, the program will automatically update the preview in your web browser.
Once you're finished editing, you can save your changes to the Markdown file in your IDE or text editor and the preview in your web browser will update automatically.
