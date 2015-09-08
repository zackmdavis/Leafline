dev: web_client/resources/public/js/client-built.js web_client/resources/public/js/chessboard-0.3.0.js \
	src/motion.rs

all: dev src/*.rs
	cargo build --release

src/motion.rs: furniture/tablemaker.py
	python $<

web_client/resources/public/js/chessboard-0.3.0.js: web_client/resources/chessboardjs-0.3.0.zip
	mkdir -p chessboard
	tar xf $< -C chessboard
	mv chessboard/js/* web_client/resources/public/js/
	mv chessboard/css/* web_client/resources/public/css/
	rm -rf chessboard

web_client/resources/chessboardjs-0.3.0.zip: tasks.py
	invoke download_statics

web_client/resources/public/js/client-built.js: web_client/resources/public/js/client.js
	node_modules/babel/bin/babel.js web_client/resources/public/js/client.js \
		--out-file web_client/resources/public/js/client-built.js

.PHONY: clean
clean:
	rm web_client/resources/chessboardjs-0.3.0.zip
	rm web_client/resources/public/js/client-built.js

