# What you need to do to hack on Leafline

### Dependencies

Essential:
* [Rust Nightly 1.8.0+](https://github.com/brson/multirust) for the main game engine
* [Leiningen](http://leiningen.org/) for the web client server
* NodeJS and [Babel](https://babeljs.io/) to compile the web client JavaScripts

Recommended:
* Python and [Invoke](http://www.pyinvoke.org/) for the build command helpers.
  * Alternatively, there's also a Makefile apparently?

### Build commands

* `inv build_furniture` will generate src/motion.rs and src/landmark.rs, which contain essesntial position tables.
* `inv compile_client` will compile the web client JavaScripts.

***XXX TODO FIXME*** finish coding Invoke tasks for all build steps and document them here

### You now might have a working environment to hack on!

* Try running the tests: `cargo test`
* Try compiling the game engine: `cargo build --release`
  * The `--release` flag turns on optimizations, which are useful even in development of this fairly CPU-intensive application.
* Try playing the console game: `cargo run --release`
* Try running the web server: `cd web_client; lein run`. The web client will be available on localhost:2882.
  * It's easy to remember because 2882 is Magnus Carlsen's peak ELO rating.
* Alternatively, `lein repl` starts the REPL, from which `(restart-server!)` starts or restarts the server.
