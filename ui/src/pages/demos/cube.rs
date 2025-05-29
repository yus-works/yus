// #[component]
// pub fn Mandelbrot() -> impl IntoView {
//     // canvas id so renderer can grab it
//     let canvas_id = "mandel-canvas";
//     use_isomorphic_effect(move || { start_cube(canvas_id); || () });
//
//     view! {
//       <h3 class="text-xl font-bold mb-2">"Mandelbrot in WebGPU"</h3>
//       <canvas id=canvas_id class="w-full h-[500px] border"></canvas>
//     }
// }
