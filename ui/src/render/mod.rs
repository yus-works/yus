#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub mod web_gpu;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub mod renderer;
