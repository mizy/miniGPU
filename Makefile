export RUST_BACKTRACE=1
export RUSTFLAGS=--cfg=web_sys_unstable_apis

triangle:
	cargo run --example triangle

window:
	cargo run --example window

image:
	cargo run --example image

camera:
	cargo run --example camera
	
wasm:
	wasm-pack build examples/wasm 

instance:
	cargo run --example instance

objloader:
	cargo run --example objloader
blinn:
	cargo run --example blinn
geometry:
	cargo run --example geometry