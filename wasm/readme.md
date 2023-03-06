# WASM

If you dont have wasm pack installed install it wit
```
cargo install wasm-pack
```


To build run
```
wasm-pack build --target web 
```

When it has finished building copy the files
```
./pkg/wasm.js -> ./web/wasm.js
./pkg/wasm_bg.wasm  -> ./web/wasm_bg.wasm
```

Then just host a local web server there and if you did everything correctly it should work.