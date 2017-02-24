# uci-rs
A library to programmatically communicate with UCI compatible chess engines

```rust
extern crate uci;

use uci::Engine;

let engine = Engine::new("/path/to/engine").unwrap();
println!("{}", engine.bestmove());
```

