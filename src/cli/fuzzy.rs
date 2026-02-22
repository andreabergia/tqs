#[derive(Clone, Copy)]
struct CommandSpec {
    canonical: &'static str,
    aliases: &'static [&'static str],
}

const COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        canonical: "create",
        aliases: &["new", "add"],
    },
    CommandSpec {
        canonical: "list",
        aliases: &["ls"],
    },
    CommandSpec {
        canonical: "info",
        aliases: &["show", "view"],
    },
    CommandSpec {
        canonical: "complete",
        aliases: &["done", "finish", "close"],
    },
    CommandSpec {
        canonical: "reopen",
        aliases: &["open"],
    },
    CommandSpec {
        canonical: "delete",
        aliases: &["remove", "rm", "del"],
    },
    CommandSpec {
        canonical: "move",
        aliases: &["rename", "mv"],
    },
];

fn prefix_match(input: &str, target: &str) -> bool {
    target.to_lowercase().starts_with(&input.to_lowercase())
}

pub fn fuzzy_match(input: &str, target: &str) -> bool {
    if input.is_empty() || target.is_empty() {
        return false;
    }

    let input_lower = input.to_lowercase();
    let target_lower = target.to_lowercase();

    let mut target_iter = target_lower.chars();

    for ch in input_lower.chars() {
        loop {
            match target_iter.next() {
                Some(target_ch) => {
                    if ch == target_ch {
                        break;
                    }
                }
                None => {
                    return false;
                }
            }
        }
    }

    true
}

fn exact_match(input: &str, target: &str) -> bool {
    input.eq_ignore_ascii_case(target)
}

fn pick_unique_shortest(candidates: &[(&'static str, usize)]) -> Option<&'static str> {
    let min_len = candidates.iter().map(|(_, len)| *len).min()?;

    let mut winner = None;
    for &(canonical, len) in candidates {
        if len != min_len {
            continue;
        }

        match winner {
            None => winner = Some(canonical),
            Some(existing) if existing == canonical => {}
            Some(_) => return None,
        }
    }

    winner
}

fn pick_unique_shortest_alias(matches: &[(&'static str, usize)]) -> Option<&'static str> {
    let mut best_per_canonical: Vec<(&'static str, usize)> = Vec::new();

    for &(canonical, alias_len) in matches {
        if let Some((_, best_len)) = best_per_canonical
            .iter_mut()
            .find(|(existing, _)| *existing == canonical)
        {
            if alias_len < *best_len {
                *best_len = alias_len;
            }
            continue;
        }

        best_per_canonical.push((canonical, alias_len));
    }

    pick_unique_shortest(&best_per_canonical)
}

fn resolve_command(input: &str) -> Option<&'static str> {
    if input.is_empty() {
        return None;
    }

    if let Some(cmd) = COMMAND_SPECS
        .iter()
        .find(|spec| exact_match(input, spec.canonical))
        .map(|spec| spec.canonical)
    {
        return Some(cmd);
    }

    let exact_alias_matches: Vec<&'static str> = COMMAND_SPECS
        .iter()
        .filter(|spec| spec.aliases.iter().any(|alias| exact_match(input, alias)))
        .map(|spec| spec.canonical)
        .collect();

    if !exact_alias_matches.is_empty() {
        let unique: Vec<(&'static str, usize)> = exact_alias_matches
            .iter()
            .map(|&canonical| (canonical, 0))
            .collect();
        return pick_unique_shortest(&unique);
    }

    let prefix_canonical_matches: Vec<(&'static str, usize)> = COMMAND_SPECS
        .iter()
        .filter(|spec| prefix_match(input, spec.canonical))
        .map(|spec| (spec.canonical, spec.canonical.len()))
        .collect();

    if let Some(cmd) = pick_unique_shortest(&prefix_canonical_matches) {
        return Some(cmd);
    }

    let fuzzy_canonical_matches: Vec<(&'static str, usize)> = COMMAND_SPECS
        .iter()
        .filter(|spec| fuzzy_match(input, spec.canonical))
        .map(|spec| (spec.canonical, spec.canonical.len()))
        .collect();

    if let Some(cmd) = pick_unique_shortest(&fuzzy_canonical_matches) {
        return Some(cmd);
    }

    let prefix_alias_matches: Vec<(&'static str, usize)> = COMMAND_SPECS
        .iter()
        .flat_map(|spec| {
            spec.aliases
                .iter()
                .filter(move |&&alias| prefix_match(input, alias))
                .map(move |&alias| (spec.canonical, alias.len()))
        })
        .collect();

    if let Some(cmd) = pick_unique_shortest_alias(&prefix_alias_matches) {
        return Some(cmd);
    }

    let fuzzy_alias_matches: Vec<(&'static str, usize)> = COMMAND_SPECS
        .iter()
        .flat_map(|spec| {
            spec.aliases
                .iter()
                .filter(move |&&alias| fuzzy_match(input, alias))
                .map(move |&alias| (spec.canonical, alias.len()))
        })
        .collect();

    pick_unique_shortest_alias(&fuzzy_alias_matches)
}

pub fn expand_command(args: Vec<String>) -> Vec<String> {
    if args.len() < 2 {
        return args;
    }

    let mut command_index = None;
    let mut i = 1;

    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with('-') {
            i += 1;
            if i < args.len() && !args[i].starts_with('-') {
                i += 1;
            }
        } else {
            command_index = Some(i);
            break;
        }
    }

    let command_index = match command_index {
        Some(idx) => idx,
        None => return args,
    };

    let first_arg = &args[command_index];
    let matched_command = resolve_command(first_arg);

    if let Some(cmd) = matched_command {
        let mut expanded_args = args.clone();
        expanded_args[command_index] = cmd.to_string();
        expanded_args
    } else {
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match_exact_match() {
        assert!(fuzzy_match("create", "create"));
        assert!(fuzzy_match("list", "list"));
    }

    #[test]
    fn test_fuzzy_match_subset_in_order() {
        assert!(fuzzy_match("l", "list"));
        assert!(fuzzy_match("ls", "list"));
        assert!(fuzzy_match("lst", "list"));
        assert!(fuzzy_match("cr", "create"));
        assert!(fuzzy_match("crt", "create"));
        assert!(fuzzy_match("cmp", "complete"));
        assert!(fuzzy_match("opn", "reopen"));
        assert!(fuzzy_match("d", "delete"));
        assert!(fuzzy_match("del", "delete"));
        assert!(fuzzy_match("i", "info"));
        assert!(fuzzy_match("inf", "info"));
        assert!(fuzzy_match("m", "move"));
        assert!(fuzzy_match("mov", "move"));
    }

    #[test]
    fn test_fuzzy_match_not_matching() {
        assert!(!fuzzy_match("rm", "delete"));
        assert!(!fuzzy_match("xyz", "create"));
        assert!(!fuzzy_match("ab", "list"));
    }

    #[test]
    fn test_fuzzy_match_empty_input() {
        assert!(!fuzzy_match("", "create"));
        assert!(!fuzzy_match("list", ""));
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        assert!(fuzzy_match("C", "create"));
        assert!(fuzzy_match("CR", "create"));
        assert!(fuzzy_match("L", "list"));
        assert!(fuzzy_match("LIST", "list"));
    }

    #[test]
    fn test_expand_command_cre() {
        let args = vec!["tqs".to_string(), "cr".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "create");
    }

    #[test]
    fn test_expand_command_l() {
        let args = vec!["tqs".to_string(), "l".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "list");
    }

    #[test]
    fn test_expand_command_list() {
        let args = vec!["tqs".to_string(), "ls".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "list");
    }

    #[test]
    fn test_expand_command_complete() {
        let args = vec!["tqs".to_string(), "cmp".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "complete");
    }

    #[test]
    fn test_expand_command_ambiguous_c() {
        let args = vec!["tqs".to_string(), "c".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "create");
    }

    #[test]
    fn test_expand_command_m() {
        let args = vec!["tqs".to_string(), "m".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "move");
    }

    #[test]
    fn test_expand_command_with_global_flags_before() {
        let args = vec![
            "tqs".to_string(),
            "--root".to_string(),
            "/path".to_string(),
            "l".to_string(),
        ];
        let expanded = expand_command(args);
        assert_eq!(expanded[3], "list");
        assert_eq!(expanded[1], "--root");
    }

    #[test]
    fn test_expand_command_with_global_flags_after() {
        let args = vec![
            "tqs".to_string(),
            "l".to_string(),
            "--root".to_string(),
            "/path".to_string(),
        ];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "list");
        assert_eq!(expanded[2], "--root");
    }

    #[test]
    fn test_expand_command_no_match() {
        let args = vec!["tqs".to_string(), "xyz".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "xyz");
    }

    #[test]
    fn test_expand_command_with_args() {
        let args = vec!["tqs".to_string(), "l".to_string(), "keyword".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "list");
        assert_eq!(expanded[2], "keyword");
    }

    #[test]
    fn test_expand_command_empty_args() {
        let args = vec!["tqs".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded.len(), 1);
    }

    #[test]
    fn test_expand_command_no_match_first_arg_is_flag() {
        let args = vec!["tqs".to_string(), "--help".to_string(), "list".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "--help");
    }

    #[test]
    fn test_expand_command_alias_new() {
        let args = vec!["tqs".to_string(), "new".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "create");
    }

    #[test]
    fn test_expand_command_alias_rename() {
        let args = vec!["tqs".to_string(), "rename".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "move");
    }

    #[test]
    fn test_expand_command_alias_remove() {
        let args = vec!["tqs".to_string(), "remove".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "delete");
    }

    #[test]
    fn test_expand_command_alias_rm() {
        let args = vec!["tqs".to_string(), "rm".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "delete");
    }

    #[test]
    fn test_expand_command_alias_show() {
        let args = vec!["tqs".to_string(), "show".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "info");
    }

    #[test]
    fn test_expand_command_alias_done() {
        let args = vec!["tqs".to_string(), "done".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "complete");
    }

    #[test]
    fn test_expand_command_alias_open() {
        let args = vec!["tqs".to_string(), "open".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "reopen");
    }

    #[test]
    fn test_expand_command_canonical_preferred_over_alias_fuzzy() {
        let args = vec!["tqs".to_string(), "c".to_string()];
        let expanded = expand_command(args);
        assert_eq!(expanded[1], "create");
    }

    #[test]
    fn test_pick_unique_shortest_ambiguous_returns_none() {
        let candidates = vec![("alpha", 3), ("beta", 3)];
        assert_eq!(pick_unique_shortest(&candidates), None);
    }
}
