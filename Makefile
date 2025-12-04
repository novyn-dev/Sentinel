BIN_PATH = target/release/sentinel_src 
PREFIX = /usr/local

all: $(BIN_PATH)

build:
	@echo "OUT_DIR: $(shell cargo metadata --format-version 1 | jq -r '.target_directory')/release/build/sentinel-*/out/"
	@ls -l target/release/build/sentinel-*/out/
	LIEF_WRAPPER_PATH=/usr/local/lib/ \
	RUSTFLAGS=-Awarnings cargo build --release

install:
	cp $(BIN_PATH) $(PREFIX)/bin
	cp c_code/exe/liblief_wrapper.so $(PREFIX)/lib
	mkdir -p $(PREFIX)/share/sentinel
	test -f $(PREFIX)/share/sentinel/passwd.db || touch $(PREFIX)/share/sentinel/passwd.db
	sudo chown $(USER) $(PREFIX)/share/sentinel/passwd.db
	sudo chmod 644 $(PREFIX)/share/sentinel/passwd.db

enable-daemon:
	set -e
	install -m 644 sentinel.service /etc/systemd/system/
	systemctl daemon-reload
