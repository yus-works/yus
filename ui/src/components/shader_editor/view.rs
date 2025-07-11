use crate::components::demo::Demo;
use crate::components::demos::utils::is_desktop;
use crate::pages::classic::classic::PassFlags;
use leptos::prelude::AnyView;
use leptos::prelude::For;
use leptos::prelude::IntoAny;
use leptos::prelude::Memo;
use leptos::prelude::event_target_checked;
use leptos::{
    IntoView, component,
    control_flow::Show,
    html::Textarea,
    prelude::{
        ClassAttribute, ElementChild, Get, NodeRef, NodeRefAttribute, OnAttribute, PropAttribute,
        RwSignal, Set, event_target_value,
    },
    view,
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Mode {
    Insert,
    Normal,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Tab {
    Vs,
    Fs,
    Ui,
}

type Handler<E> = Box<dyn FnMut(E)>;

use super::utils::keydown;

#[component]
fn OptionsPanel(pass_flags: PassFlags) -> impl IntoView {
    let items = move || pass_flags.iter();

    view! {
        <div class="flex flex-col gap-2">
            <For
                each=items
                key=|pair: &(String, RwSignal<bool>)| pair.0.clone()
                children=move |(label, sig): (String, RwSignal<bool>)| {
                    view! {
                        <label class="flex items-center gap-2">
                            <input
                                type="checkbox"
                                prop:checked=move || sig.get()
                                on:change=move |ev| sig.set(event_target_checked(&ev))
                            />
                            <span class="text-text">{ label }</span>
                        </label>
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn TabBar(active_tab: RwSignal<Tab>, ui_enabled: Memo<bool>) -> impl IntoView {
    // helper returns an AnyView, so its concrete type stays small
    let mk_btn = move |label: &'static str, tab: Tab| -> AnyView {
        let active_tab = active_tab.clone();

        let on_click: Handler<web_sys::MouseEvent> = Box::new(move |_| active_tab.set(tab));

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
            {
                move || {
                    if ui_enabled.get() {
                        mk_btn("UI", Tab::Ui)
                    } else {
                        view!{}.into_any()
                    }
                }
            }
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
pub fn ShaderEditor(
    vs_src: RwSignal<String>,
    fs_src: RwSignal<String>,

    pass_flags: PassFlags,
    selected_demo: RwSignal<Demo>,
) -> impl IntoView {
    let vim_enabled = is_desktop();

    let active_tab = RwSignal::new(Tab::Vs);

    let ui_enabled = Memo::new(move |_| selected_demo.get() == Demo::Animals);

    let textarea_ref = NodeRef::<Textarea>::new();

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
            Mode::Insert => format!("-- INSERT -- [{tab_lbl}]"),
            Mode::Normal => format!("-- NORMAL -- [{tab_lbl}]"),
        }
    };

    view! {
        <div class="w-full h-[40rem] flex flex-col" on:click=focus_textarea>
            <TabBar active_tab ui_enabled />

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

            <Show
                when=move || (active_tab.get() == Tab::Ui) && (ui_enabled.get())
            >
                {
                    let flags_handle = pass_flags.clone();
                    view! { <OptionsPanel pass_flags=flags_handle /> }.into_any()
                }
            </Show>

            <Show when=move || vim_enabled>
                { view! { <StatusBar status/> }.into_any() }
            </Show>
        </div>
    }
}
