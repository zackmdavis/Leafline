# What you need to do to hack on Leafline

### Getting your life in order

Your life is not in order. You're probably not running python3, you probably don't have Rust installed. christ you might not even have Clojure installed. Get it together, folks.

* You must install the Rust nightly build. I don't care which night. Just a recent one.
* You must install clojure and lein, which I forget what lein is even short for, and I don't remember how to install it, because, unlike you, __my life is together__, and I did that months ago. or years. I don't remember.
* install python3
* `virtualenv --python=python3.4 .`
* `source bin/activate` obviously you knew that one
* `pip install -r requirements.txt`
* holy shit you need `npm` for this, that's insane, i thought this was the future.
  * go here https://nodejs.org/en/ and do what it says
  * then run `npm install babel`
  * we are SERIOUS PEOPLE here, don't do that `-g` crap, the same way you didn't `sudo` up your pip earlier.
* run `make all`
  * this might take a little while because it has to do a full release build. use `make` or `make dev` for every day things


# you now might have a working environment to hack on!

* try running the tests: `cargo test`
* try running the web server: `cd web_client; lein run`. it'll open up in localhost:2882
  * It's easy to remember because 2882 is Magnus Carlsen's peak ELO rating.
* Alternatively, `lein repl` starts the REPL, from which `(restart-server!)` starts or restarts the server.
