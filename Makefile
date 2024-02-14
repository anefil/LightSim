.PHONY: clean default wasm_debug_run wasm_build_debug windows_build_debug clean target_list
MAKEFLAGS += -j2

#Really, should be ENV, but I don't care
BROWSER_LOCATION := C:\Users\eee\AppData\Local\Google\Chrome SxS\Application\chrome.exe
#This should be fetched from elsewhere, too
ADDRESS := http://127.0.0.1:301/index.html

SHADER_SRC_DIR := shaders
#for wasm
SHADER_DEST_DIR := web/shaders

SHADER_SRC_FILES := $(wildcard $(SHADER_SRC_DIR)/*)

# default: windows_debug_run wasm_debug_run 
default: windows_debug_run
release: windows_release_run

# compare: compare_windows
# compare_windows: preq
# 	cargo build --target x86_64-pc-windows-msvc --release
# 	mv .\target\x86_64-pc-windows-msvc\release\rs_wgpu_cube.exe .\target\compare
# 	CMD /C start ren ".\target\compare\rs_wgpu_cube.exe" old.exe
# 	cargo build --target x86_64-pc-windows-msvc --features new --release
# 	mv .\target\x86_64-pc-windows-msvc\release\rs_wgpu_cube.exe .\target\compare
# 	CMD /C start ren .\target\compare\rs_wgpu_cube.exe new.exe
# 	CMD /C start CMD /K  .\target\compare\old.exe
# 	CMD /C start CMD /K  .\target\compare\new.exe

# preq:
# 	mkdir -p .\target\compare


wasm_debug_run: wasm_build_debug
	CMD /C start CMD /K  simple-http-server .\web --ip 127.0.0.1 -p 301
	'${BROWSER_LOCATION}' ${ADDRESS}

windows_release_run: windows_build_debug
	cargo run --target x86_64-pc-windows-msvc --release ${FEATURES}

windows_debug_run: windows_build_debug
	CMD /C start  CMD /K  .\target\x86_64-pc-windows-msvc\debug\rs_wgpu_cube.exe


wasm_build_debug: wasm_copy_shaders
	cargo build --target wasm32-unknown-unknown 
	wasm-bindgen --out-dir web\wasm --web .\target\wasm32-unknown-unknown\debug\rs_wgpu_cube.wasm

wasm_copy_shaders: 
	cp -r $(SHADER_SRC_FILES) $(SHADER_DEST_DIR)

windows_build_debug:
	cargo build --target x86_64-pc-windows-msvc



# utils
clean:
	rm -rf target

target_list:
	rustc --print target-list

install:
	cargo install -f wasm-bindgen-cli
	cargo install -f simple-http-server
