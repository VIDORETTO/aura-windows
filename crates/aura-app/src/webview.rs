//! What the native web view may show: Aura is a desktop app, not a web page.
//! The right-click menu keeps only editing actions (no "Save as", "Print",
//! "Back", "Reload", "Inspect"…); the shell applies this to every window.

/// One entry of the native context menu, as the web view reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEntry<'a> {
    Command(&'a str),
    Separator,
    /// Submenus (share, web capture…) never fit a desktop app.
    Submenu(&'a str),
}

/// Editing actions kept in inputs, on selected text and on images.
const KEEP: &[&str] = &[
    "undo",
    "redo",
    "cut",
    "copy",
    "paste",
    "pasteAndMatchStyle",
    "selectAll",
    "emoji",
    "copyImage",
    "spellCheck",
];

/// Indices of the entries to keep, in order, without leading, trailing or
/// doubled separators. Empty → the menu is not shown at all.
pub fn context_menu_keep(entries: &[MenuEntry<'_>]) -> Vec<usize> {
    let mut keep: Vec<usize> = Vec::new();
    let mut pending_separator = None;
    for (i, e) in entries.iter().enumerate() {
        match e {
            MenuEntry::Command(name) if KEEP.contains(name) => {
                if let Some(s) = pending_separator.take()
                    && !keep.is_empty()
                {
                    keep.push(s);
                }
                keep.push(i);
            }
            MenuEntry::Separator => pending_separator = Some(i),
            _ => {}
        }
    }
    keep
}

#[cfg(test)]
mod tests {
    use super::MenuEntry::{Command as C, Separator as S, Submenu as Sub};
    use super::*;

    #[test]
    fn page_background_menu_disappears() {
        // Right click on empty space: WebView2's page menu.
        let page = [
            C("back"),
            C("forward"),
            C("reload"),
            S,
            C("saveAs"),
            C("print"),
            Sub("share"),
            S,
            C("inspectElement"),
        ];
        assert_eq!(context_menu_keep(&page), Vec::<usize>::new());
    }

    #[test]
    fn input_menu_keeps_editing_with_tidy_separators() {
        let input = [
            C("emoji"),
            S,
            C("undo"),
            C("redo"),
            S,
            C("cut"),
            C("copy"),
            C("paste"),
            S,
            C("selectAll"),
            S,
            C("inspectElement"),
        ];
        assert_eq!(
            context_menu_keep(&input),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
    }

    #[test]
    fn selected_text_keeps_copy_only() {
        let selection = [
            C("copy"),
            S,
            Sub("webCapture"),
            C("print"),
            S,
            C("inspectElement"),
        ];
        assert_eq!(context_menu_keep(&selection), vec![0]);
    }
}
