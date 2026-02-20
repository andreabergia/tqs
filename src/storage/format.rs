#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedTaskFile {
    pub frontmatter: String,
    pub body: String,
}

pub fn parse_task_file(input: &str) -> ParsedTaskFile {
    ParsedTaskFile {
        frontmatter: String::new(),
        body: input.to_string(),
    }
}
