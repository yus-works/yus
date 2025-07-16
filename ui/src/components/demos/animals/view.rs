use leptos::{
    IntoView,
    prelude::{ClassAttribute, ElementChild, Get, GlobalAttributes, RwSignal, Show},
    view,
};

use crate::components::demos::utils::WebGPUNotSupportedMsg;
use super::main::CANVAS_ID;

pub(crate) fn canvas(gpu_support: RwSignal<bool>, show_hint: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class="relative w-full group">
          <Show
            when=move || matches!(gpu_support.get(), true)
            fallback=move || view! { <WebGPUNotSupportedMsg/> }
          >

          <canvas
            id=CANVAS_ID
            width="864"
            height="1024"
            class="w-full h-full object-cover touch-none select-none"
          ></canvas>

          <Show when=move || show_hint.get()>
              <div id="hint"
                   class="pointer-events-none absolute inset-0 flex flex-col items-center justify-center
                          bg-white/70 backdrop-blur-sm text-surface text-sm gap-2
                          transition-opacity duration-500
                          group-hover:opacity-0">
                ""
                <p>"Click & drag to move the animal ✋"</p>
                <strong>"Click to hide this hint"</strong>
              </div>
          </Show>

          </Show>
        </div>
    }
}
