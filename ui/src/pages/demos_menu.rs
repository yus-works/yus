use leptos::*;
use leptos::prelude::ElementChild;
use leptos::prelude::ClassAttribute;

#[component]
pub fn DemosMenu() -> impl IntoView {
    view! {
      <h2 class="text-xl font-bold mb-4">"WebGPU demos"</h2>
      <ul class="list-disc pl-6">
        <li><a href="/demos/mandelbrot">"Mandelbrot"</a></li>
        <li><a href="/demos/cube">"3-D cube"</a></li>
      </ul>
    }
}
