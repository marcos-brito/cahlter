# Cahlter

Cahlter is a website generator. It’s somewhat similar to [gitbook](https://www.gitbook.com/), [mkdocs](https://www.mkdocs.org/) or [mdBook](https://rust-lang.github.io/mdBook/).
It has easy theming, client-side search, and a nice CLI. Check the [docs]() to see what you can do ⚡.

> The docs were generated with Cahlter itself.

# Installation

At the moment, there are no plans to package this for any package manager, but you can build it using the [repository](https://github.com/marcos-brito/cahlter):

First, clone the repo:

```bash
git clone https://github.com/marcos-brito/cahlter && cd cahlter
```

then run:

```bash
cargo build --release
```

> Make sure you have Cargo installed. You can read about it [here](https://doc.rust-lang.org/cargo/).

If you want, you can copy the binary to something in your PATH:

```bash
cp ./target/release/cahlter ~/.local/bin
```

# Architecture\*

Cahlter is a static website generator. The whole idea is to take a bunch of text files and generate some HTML.

The main concept in Cahlter is the vault. This is what you need to know about it:

- It’s where the text files are placed.
- It’s, by default, where the website will be built.
- It’s simply a directory in the file system.

> Yes, I borrowed the name "vault" from Obsidian 🫣

The text files within the vault are read to create a summary. A summary represents all the text content inside the vault. Each item within the summary can be either:

- A chapter: It holds a name, a path, a number, and subchapters.
- A section: It contains only a title.

The summary is always the output of a summarizer. Currently, it can be generated using the file tree or a summary file.

After that, the renderers will take all the chapters and generate something. At the moment, there is only an Askama renderer that uses the Askama template engine. After generation, the output is written to a file within the build directory.

> Templates, scripts, and CSS are static data. So they will always be there

Everything is actually pretty straightforward and simple. Here is a short summary for everything:

**Text files** -> **Summarizer** -> **Renderer** -> **HTML**
