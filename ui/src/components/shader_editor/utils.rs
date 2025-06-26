use leptos::html::Textarea;
use leptos::prelude::NodeRef;
use leptos::prelude::Get;
use leptos::prelude::Set;
use leptos::prelude::RwSignal;

use super::view::Mode;

pub(crate) fn keydown(
    vim_enabled: bool,
    mode: RwSignal<Mode>,
    textarea_ref: NodeRef<Textarea>,
) -> impl Fn(web_sys::KeyboardEvent) + 'static {
    move |ev: web_sys::KeyboardEvent| {
        if !vim_enabled {
            return;
        }

        match mode.get() {
            Mode::Insert => {
                if ev.key() == "Escape" || ev.key() == "Esc" {
                    mode.set(Mode::Normal);
                    update_block_cursor(&textarea_ref, Mode::Normal);
                    ev.prevent_default();
                }
            }
            Mode::Normal => {
                match ev.key().as_str() {
                    "i" => {
                        mode.set(Mode::Insert);
                        update_block_cursor(&textarea_ref, Mode::Insert);
                        ev.prevent_default();
                    }
                    "h" | "j" | "k" | "l" | "w" | "b" | "0" | "$" => {
                        vim_motion(&textarea_ref, &ev.key());
                        update_block_cursor(&textarea_ref, Mode::Normal);
                        ev.prevent_default();
                    }
                    _ => ev.prevent_default(),
                }
            }
        }
    }
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
    let _end = t.selection_end().unwrap().unwrap();

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

