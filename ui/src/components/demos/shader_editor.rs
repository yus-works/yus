use leptos::{
    component, html::Textarea, leptos_dom::logging::console_log, prelude::{
        event_target_value, signal, ClassAttribute, ElementChild, Get, NodeRef, NodeRefAttribute, OnAttribute, PropAttribute, RwSignal, Set
    }, view, IntoView
};

#[derive(Copy, Clone, PartialEq, Eq)]
enum Mode {
    Insert,
    Normal,
}

fn update_block_cursor(textarea: &NodeRef<Textarea>, mode: Mode) {
    let Some(t) = textarea.get() else { return };

    let pos = t.selection_start().unwrap().unwrap();

    match mode {
        Mode::Insert => {
            // collapse selection
            t.set_selection_end(Some(pos)).ok();
        }
        Mode::Normal => {
            // select the character under (after) the cursor
            t.set_selection_end(Some(pos + 1)).ok();
        }
    }
}

fn vim_motion(textarea: &NodeRef<Textarea>, key: &str) {
    let Some(t) = textarea.get() else {
        return;
    };

    let start = t.selection_start().unwrap().unwrap();

    // TODO: add basic selections and line edits and shit
    let end = t.selection_end().unwrap().unwrap();

    let mut pos = start; // treat both ends the same

    let value = t.value();
    let len = value.len() as u32;

    match key {
        "h" => { if pos > 0 { pos -= 1; } }
        "l" => { if pos < len { pos += 1; } }
        "0" => pos = 0, "$" => pos = len,
        "w" => { // next word
            if let Some(off) = value[pos as usize..].find(|c: char| c.is_alphanumeric()) {
                pos += off as u32;
                if let Some(off2) = value[pos as usize..].find(|c: char| !c.is_alphanumeric()) {
                    pos += off2 as u32;
                } else {
                    pos = len;
                }
            }
        }
        "b" => { // previous word
            if pos > 0 {
                let rev = value[..pos as usize].chars().rev();
                let mut seen_word = false;
                for (i, c) in rev.enumerate() {
                    if c.is_alphanumeric() {
                        seen_word = true;
                    } else if seen_word {
                        pos -= i as u32;
                        break;
                    }
                }
            }
        }
        "j" | "k" => {
            // line-wise: need split on '\n'
            let lines: Vec<&str> = value.split('\n').collect();
            let mut cur = 0u32;
            let mut row = 0usize;
            for (i, line) in lines.iter().enumerate() {
                if cur + line.len() as u32 + 1 > pos {
                    row = i;
                    break;
                }
                cur += line.len() as u32 + 1; // +1 for '\n'
            }
            let col = pos - cur;
            if key == "j" && row + 1 < lines.len() {
                pos = cur + lines[row].len() as u32 + 1 + col.min(lines[row + 1].len() as u32);
            }
            if key == "k" && row > 0 {
                let prev_len = lines[row - 1].len() as u32;
                let prev_start = cur - (prev_len + 1);
                pos = prev_start + col.min(prev_len);
            }
        }
        _ => {}
    }
    t.set_selection_start(Some(pos)).ok();
    t.set_selection_end(Some(pos)).ok();
}

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

    let textarea_ref = NodeRef::<Textarea>::new();

    // give the outer div a click handler so the whole card grabs focus
    let focus_textarea = move |_| {
        if let Some(t) = textarea_ref.get() {
            t.focus().ok();
            console_log("Hello gamers");
        }
    };

    let (mode, set_mode) = signal(Mode::Normal);

    // human-readable status line
    let status = move || match mode.get() {
        Mode::Insert => format!("-- INSERT --   [{}]", active_tab.get()),
        Mode::Normal => format!("-- NORMAL --   [{}]", active_tab.get()),
    };

    let keydown = move |ev: web_sys::KeyboardEvent| {
        match mode.get() {
            Mode::Insert => {
                if ev.key() == "Escape" || ev.key() == "Esc" {
                    set_mode.set(Mode::Normal);
                    update_block_cursor(&textarea_ref, Mode::Normal);
                    ev.prevent_default();
                }
            }

            Mode::Normal => {
                match ev.key().as_str() {
                    "i" => {
                        set_mode.set(Mode::Insert);
                        update_block_cursor(&textarea_ref, Mode::Insert);
                        ev.prevent_default();
                    }
                    "h" | "j" | "k" | "l"
                    | "w" | "b" | "0" | "$" => {
                        vim_motion(&textarea_ref, &ev.key());
                        update_block_cursor(&textarea_ref, Mode::Normal);
                        ev.prevent_default();
                    }
                    _ => {
                        ev.prevent_default();
                    }
                }
            }
        }
    };

    view! {
        <div class="w-full h-[22rem] flex flex-col" on:click=focus_textarea>
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
                class=move || format!(
                    "flex-1 bg-surface text-text text-xs \
                     p-4 font-mono rounded-xl resize-none \
                     border border-transparent focus:border-gray-300 \
                     focus:outline-none focus:ring-1 focus:ring-gray-400 focus:ring-opacity-50 \
                     selection:bg-text selection:text-surface \
                     {}", if mode.get() == Mode::Normal { "caret-transparent" } else { "caret-visible" }
                )
                prop:value=active_src
                on:input=on_input
                on:keydown=keydown
                node_ref=textarea_ref
            />

            /* ---------- status bar ---------- */
            <div class="h-6 px-3 text-xs flex items-center
                        bg-neutral-dark text-text border-t border-gray-700 select-none">
                { status }
            </div>
        </div>
    }

}
