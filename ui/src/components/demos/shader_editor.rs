use leptos::{component, prelude::{event_target_value, signal, ElementChild, ClassAttribute, Get, OnAttribute, PropAttribute, RwSignal, Set}, view, IntoView};

#[component]
pub fn ShaderEditor(vs_src: RwSignal<String>, fs_src: RwSignal<String>) -> impl IntoView {
    let (active_tab, set_active_tab) = signal("VS");

    let active_src = move || {
        if active_tab.get() == "VS" {
            vs_src.get()
        } else {
            fs_src.get()
        }
    };

    let on_input = move |ev| {
        let val = event_target_value(&ev);
        if active_tab.get() == "VS" {
            vs_src.set(val);
        } else {
            fs_src.set(val);
        }
    };

    view! {
        <div class="w-full h-[22rem]">
            // Tab buttons
            <div class="flex">
                <button
                    class=move || format!(
                        "px-4 py-1 border-b-2 {}",
                        if active_tab.get() == "VS" {
                            "border-white text-white"
                        } else {
                            "border-transparent text-gray-400"
                        }
                    )
                    on:click=move |_| set_active_tab.set("VS")
                >
                    "VS"
                </button>
                <button
                    class=move || format!(
                        "px-4 py-1 border-b-2 {}",
                        if active_tab.get() == "FS" {
                            "border-white text-white"
                        } else {
                            "border-transparent text-gray-400"
                        }
                    )
                    on:click=move |_| set_active_tab.set("FS")
                >
                    "FS"
                </button>
            </div>

            <textarea
              class="w-full h-[22rem]
                     bg-surface text-text text-xs
                     p-4 font-mono rounded-xl resize-none
                     border border-transparent
                     focus:border-gray-300
                     focus:outline-none
                     focus:ring-1
                     focus:ring-gray-400
                     focus:ring-opacity-50
              "
              prop:value=active_src
              on:input=on_input
            />

        </div>
    }
}
