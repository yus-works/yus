use leptos::{prelude::{AnyView, IntoAny, RwSignal, Get, Set}, view, IntoView};
use leptos::prelude::ElementChild;
use leptos::prelude::ClassAttribute;
use leptos::prelude::OnAttribute;

use crate::render::renderer::vertex::Vertex;

use super::demos::{animals::Animals, cube::CubePlanet, frag_intro::FragIntro};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Demo {
    CubePlanet,
    Animals,
    FragIntro
}

const CUBE_VS: &str = include_str!("../render/renderer/shaders/cube.vert.wgsl");
const CUBE_FS: &str = include_str!("../render/renderer/shaders/cube.frag.wgsl");

const FRAG_VS: &str = include_str!("../render/renderer/shaders/frag_intro.vert.wgsl");
const FRAG_FS: &str = include_str!("../render/renderer/shaders/frag_intro.frag.wgsl");

const FISH_VS: &str = include_str!("../render/renderer/shaders/fish.vert.wgsl");
const FISH_FS: &str = include_str!("../render/renderer/shaders/fish.frag.wgsl");

impl Demo {
    pub fn label(&self) -> &'static str {
        match self {
            Demo::Animals => "Procedurally Animated Animals",
            Demo::CubePlanet => "Cube Planet Visualizer",
            Demo::FragIntro => "Fragment Shader Intro",
        }
    }
    pub fn shaders(&self) -> (&'static str, &'static str) {
        match self {
            Demo::Animals => (FISH_VS, FISH_FS),
            Demo::CubePlanet => (CUBE_VS, CUBE_FS),
            Demo::FragIntro => (FRAG_VS, FRAG_FS),
        }
    }

    /// Returns a View that mounts the proper canvas component
    pub fn canvas(self, vs_src: RwSignal<String>, fs_src: RwSignal<String>) -> AnyView {
        match self {
            Demo::Animals => view! { <Animals vs_src=vs_src fs_src=fs_src/> }.into_any(),
            Demo::CubePlanet => view! { <CubePlanet vs_src=vs_src fs_src=fs_src/> }.into_any(),
            Demo::FragIntro => view! { <FragIntro vs_src=vs_src fs_src=fs_src/> }.into_any(),
        }
    }
}

pub trait DemoTab {
    fn labelled_button(self, sel: RwSignal<Demo>) -> impl IntoView;
}

impl DemoTab for Demo {
    fn labelled_button(self, sel: RwSignal<Demo>) -> impl IntoView {
        view! {
            <li>
                <button
                    class="tab px-3 py-1"
                    // “active” class toggles styling
                    class=("active",  move || sel.get() == self)
                    // update selection on click
                    on:click=move |_| sel.set(self)
                >
                    {self.label()}
                </button>
            </li>
        }
    }
}

pub fn make_pipeline_with_topology(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    topology: wgpu::PrimitiveTopology,
    vs_src: &str,
    fs_src: &str,
) -> wgpu::RenderPipeline {
    let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("vs shader with custom topology"),
        source: wgpu::ShaderSource::Wgsl(vs_src.into()),
    });

    let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fs shader with custom topology"),
        source: wgpu::ShaderSource::Wgsl(fs_src.into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("empty layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("custom strip pipeline"),
        layout: Some(&layout),
        cache: None,
        vertex: wgpu::VertexState {
            module: &vs,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
    })
}
