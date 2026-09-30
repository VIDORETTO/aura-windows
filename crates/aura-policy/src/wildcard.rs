//! Case-insensitive wildcard matching with `*` (any run) and `?` (one char).

pub fn matches(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.to_lowercase().chars().collect();
    let t: Vec<char> = text.to_lowercase().chars().collect();
    let (mut pi, mut ti) = (0usize, 0usize);
    let (mut star, mut mark) = (None::<usize>, 0usize);
    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = ti;
            pi += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn wildcard_cases() {
        assert!(matches("*.exe", "Notepad.EXE"));
        assert!(matches("LastPass*.exe", "lastpassbroker.exe"));
        assert!(matches("*Jira*", "PROJ-1 - Jira - Chrome"));
        assert!(matches("a?c", "abc"));
        assert!(!matches("a?c", "ac"));
        assert!(!matches("code.exe", "vscode.exe"));
        assert!(matches("*", ""));
        assert!(matches("*Navegação anônima*", "Google - NAVEGAÇÃO ANÔNIMA"));
    }
}
