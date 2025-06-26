use crate::components::demos::utils::is_desktop;
use leptos::prelude::AnyView;
use leptos::prelude::IntoAny;
use leptos::prelude::ReadSignal;
use leptos::{
    IntoView, component,
    control_flow::Show,
    html::Textarea,
    prelude::{
        ClassAttribute, ElementChild, Get, NodeRef, NodeRefAttribute, OnAttribute, PropAttribute,
        RwSignal, Set, event_target_value, signal,
    },
    view,
};
use wasm_bindgen::JsCast;

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Mode {
    Insert,
    Normal,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Tab {
    Vs,
    Fs,
    Ui, // <-- NEW third tab
}

type Handler<E> = Box<dyn FnMut(E)>;

use crate::reactive::signal;

use super::utils::keydown;


#[component]
fn OptionsPanel(
    opacity: signal::RwSignal<f32>,
    show_grid: signal::RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex-1 p-4 space-y-6 bg-surface text-text rounded-xl overflow-auto">
            <div class="flex items-center space-x-3">
                <span class="w-28">Opacity</span>
                <input
                    class="flex-1"
                    type="range" min="0" max="1" step="0.01"
                    prop:value=move || opacity.get().to_string()
                    on:input=move |ev| {
                        opacity.set(event_target_value(&ev).parse().unwrap_or(0.0))
                    }
                />
                <span class="w-10 text-right">{move || format!("{:.2}", opacity.get())}</span>
            </div>

            <label class="flex items-center space-x-2">
                <input
                    type="checkbox"
                    prop:checked=show_grid
                    on:change=move |ev| {
                        let checked = ev
                            .unchecked_into::<web_sys::HtmlInputElement>()
                            .checked();
                        show_grid.set(checked);
                    }
                />
                <span>Show grid</span>
            </label>
        </div>
    }
}

#[component]
pub fn TabBar(
    active_tab: RwSignal<Tab>,
) -> impl IntoView {
    // helper returns an AnyView, so its concrete type stays small
    let mk_btn = |label: &'static str, tab: Tab| -> AnyView {
        let active_tab = active_tab.clone();

        let on_click: Handler<web_sys::MouseEvent> = 
            Box::new(move |_| active_tab.set(tab));

        view! {
            <button
                class=move || format!(
                    "px-4 py-1 border-b-2 transition-colors {}",
                    if active_tab.get() == tab {
                        "border-white text-white"
                    } else {
                        "border-transparent text-gray-400 hover:text-gray-200"
                    }
                )
                on:click=on_click
            >
                { label }
            </button>
        }
        .into_any()
    };

    view! {
        <div class="flex space-x-1">
            { mk_btn("VS", Tab::Vs) }
            { mk_btn("FS", Tab::Fs) }
            { mk_btn("UI", Tab::Ui) }
        </div>
    }
}

#[component]
fn StatusBar(status: impl Fn() -> String + Send + 'static) -> impl IntoView {
    view! {
        <div class="h-6 px-3 text-xs flex items-center bg-neutral-dark text-text \
                    border-t border-gray-700 select-none">
            { status }
        </div>
    }
}

#[component]
fn CodeArea(
    tab: RwSignal<Tab>,
    vs_src: RwSignal<String>,
    fs_src: RwSignal<String>,

    on_input: Handler<web_sys::Event>,
    on_keydown: Handler<web_sys::KeyboardEvent>,

    mode: RwSignal<Mode>,

    /// NodeRef so the parent can call `.focus()` etc.
    textarea_ref: NodeRef<Textarea>,
) -> impl IntoView {
    view! {
        <textarea
            class=move || format!(
                "flex-1 bg-surface text-text text-xs p-4 font-mono rounded-xl resize-none \
                 border border-transparent focus:border-gray-300 focus:outline-none \
                 focus:ring-1 focus:ring-gray-400 focus:ring-opacity-50 \
                 selection:bg-text selection:text-surface {}",
                if mode.get() == Mode::Normal {
                    "caret-transparent"
                } else {
                    "caret-visible"
                }
            )
            prop:value = move || match tab.get() {
                Tab::Vs => vs_src.get(),
                Tab::Fs => fs_src.get(),
                Tab::Ui => String::new(),
            }

            on:input=on_input
            on:keydown=on_keydown
            node_ref=textarea_ref
        />
    }
}

#[component]
pub fn ShaderEditor(vs_src: RwSignal<String>, fs_src: RwSignal<String>) -> impl IntoView {
    let vim_enabled = is_desktop();

    let active_tab = RwSignal::new(Tab::Vs);
    let textarea_ref = NodeRef::<Textarea>::new();

    let opacity = RwSignal::new(0.5f32);
    let show_grid = RwSignal::new(true);

    let focus_textarea = move |_| {
        if let Some(t) = textarea_ref.get() {
            t.focus().ok();
        }
    };

    let mode = RwSignal::new(Mode::Normal);

    let status = move || {
        let tab_lbl = match active_tab.get() {
            Tab::Vs => "VS",
            Tab::Fs => "FS",
            Tab::Ui => "UI",
        };
        match mode.get() {
            Mode::Insert => format!("-- INSERT --   [{tab_lbl}]"),
            Mode::Normal => format!("-- NORMAL --   [{tab_lbl}]"),
        }
    };

    view! {
            <div class="w-full h-[40rem] flex flex-col" on:click=focus_textarea>
                <TabBar active_tab />

                <Show when=move || active_tab.get() != Tab::Ui>
                    {   // these closures must be Fn, so build fresh handlers every call
                        let key_handler: Handler<web_sys::KeyboardEvent> =
                            Box::new(keydown(vim_enabled, mode, textarea_ref.clone()));

                        let on_input: Handler<web_sys::Event> = Box::new(move |ev| {
                            let val = event_target_value(&ev);
                            match active_tab.get() {
                                Tab::Vs => vs_src.set(val),
                                Tab::Fs => fs_src.set(val),
                                Tab::Ui => (),
                            }
                        });

                        view! {
                            <CodeArea
                                vs_src = vs_src
                                fs_src = fs_src
                                tab = active_tab
                                textarea_ref = textarea_ref.clone()
                                mode = mode

                                on_input = on_input
                                on_keydown = key_handler
                            />
                        }.into_any()
                    }
                </Show>

                <Show when=move || active_tab.get() == Tab::Ui>
                    { view! { <OptionsPanel opacity show_grid/> }.into_any() }
                </Show>

                <Show when=move || vim_enabled>
                    { view! { <StatusBar status/> }.into_any() }
                </Show>
            </div>
        }
}
