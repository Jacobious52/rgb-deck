
build:
	cargo build

install: build
	elf2uf2-rs ./target/thumbv6m-none-eabi/debug/rgb-deck ./target/thumbv6m-none-eabi/debug/rgb-deck.uf2
	sudo ./install.sh ./target/thumbv6m-none-eabi/debug/rgb-deck.uf2
