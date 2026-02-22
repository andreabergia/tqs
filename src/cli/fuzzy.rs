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

pub fn expand_command(args: Vec<String>) -> Vec<String> {
    const COMMANDS: &[&str] = &[
        "create", "list", "info", "complete", "reopen", "delete", "move",
    ];

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

    let matched_command = COMMANDS.iter().find(|&cmd| fuzzy_match(first_arg, cmd));

    if let Some(&cmd) = matched_command {
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
}
