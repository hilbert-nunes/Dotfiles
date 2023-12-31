use std::fmt::Write as _;

pub(crate) fn fmt_leading(leading: &mut String, indent: usize, no_comments: bool) {
    if leading.is_empty() {
        return;
    }
    let mut result = String::new();
    if !no_comments {
        let input = leading.trim();
        let kdl_parser = crate::parser::KdlParser { full_input: input };
        let comments = kdl_parser
            .parse(crate::parser::leading_comments(&kdl_parser))
            .expect("invalid leading text");
        for line in comments {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                writeln!(result, "{:indent$}{}", "", trimmed, indent = indent).unwrap();
            }
        }
    }
    write!(result, "{:indent$}", "", indent = indent).unwrap();
    *leading = result;
}

pub(crate) fn fmt_trailing(decor: &mut String, no_comments: bool) {
    if decor.is_empty() {
        return;
    }
    *decor = decor.trim().to_string();
    let mut result = String::new();
    if !no_comments {
        let input = &*decor;
        let kdl_parser = crate::parser::KdlParser { full_input: input };
        let comments = kdl_parser
            .parse(crate::parser::trailing_comments(&kdl_parser))
            .expect("invalid trailing text");
        for comment in comments {
            result.push_str(comment);
        }
    }
    *decor = result;
}
