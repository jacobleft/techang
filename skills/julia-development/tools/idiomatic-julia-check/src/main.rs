use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use fatou_parser::parser::parse;
use fatou_parser::syntax::{SyntaxKind, SyntaxNode};

#[derive(Debug)]
struct Finding {
    offset: usize,
    message: String,
}

fn reject(findings: &mut Vec<Finding>, node: &SyntaxNode, message: impl Into<String>) {
    findings.push(Finding {
        offset: u32::from(node.text_range().start()) as usize,
        message: message.into(),
    });
}

fn child_nodes(node: &SyntaxNode) -> Vec<SyntaxNode> {
    node.children().collect()
}

fn has_direct_token(node: &SyntaxNode, kind: SyntaxKind) -> bool {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .any(|token| token.kind() == kind)
}

fn is_name(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::NAME
}

fn is_type_reference(node: &SyntaxNode) -> bool {
    match node.kind() {
        SyntaxKind::NAME => true,
        SyntaxKind::CURLY_EXPR => {
            let children = child_nodes(node);
            children.len() == 2
                && is_type_reference(&children[0])
                && children[1].kind() == SyntaxKind::ARG_LIST
                && children[1].children().all(|argument| {
                    argument.kind() == SyntaxKind::ARG
                        && child_nodes(&argument)
                            .first()
                            .is_some_and(is_type_reference)
                })
        }
        SyntaxKind::BINARY_EXPR => {
            let operands = child_nodes(node);
            has_direct_token(node, SyntaxKind::SUBTYPE)
                && operands.len() == 2
                && operands.iter().all(is_type_reference)
        }
        _ => false,
    }
}

fn is_typed_noun(node: &SyntaxNode) -> bool {
    if node.kind() != SyntaxKind::TYPE_ANNOTATION {
        return false;
    }
    let children = child_nodes(node);
    children.len() == 2 && is_name(&children[0]) && is_type_reference(&children[1])
}

fn verb_name(call: &SyntaxNode) -> Option<String> {
    let callee = child_nodes(call).into_iter().next()?;
    if callee.kind() != SyntaxKind::NAME {
        return None;
    }
    let name = callee.text().to_string();
    let stem = name.strip_suffix('!').unwrap_or(&name);
    let first = stem.chars().next()?;
    (!first.is_uppercase()).then_some(name)
}

fn is_mutating_call(call: &SyntaxNode) -> bool {
    verb_name(call).is_some_and(|name| name.ends_with('!'))
}

fn validate_call(findings: &mut Vec<Finding>, call: &SyntaxNode, typed_allowed: bool) {
    if verb_name(call).is_none() {
        reject(findings, call, "call must use an unqualified verb name");
        return;
    }

    let children = child_nodes(call);
    let Some(arguments) = children
        .iter()
        .find(|child| child.kind() == SyntaxKind::ARG_LIST)
    else {
        reject(findings, call, "verb call must have an argument list");
        return;
    };

    let arguments: Vec<_> = arguments.children().collect();
    if arguments.is_empty() {
        reject(findings, call, "verb must act on at least one noun");
        return;
    }
    if arguments
        .iter()
        .any(|argument| argument.kind() != SyntaxKind::ARG)
    {
        reject(
            findings,
            call,
            "keyword and parameter arguments are not allowed",
        );
        return;
    }

    let expressions: Vec<_> = arguments
        .iter()
        .filter_map(|argument| child_nodes(argument).into_iter().next())
        .collect();
    if expressions.len() != arguments.len() {
        reject(findings, call, "every argument must contain one noun");
        return;
    }

    let all_typed = expressions.iter().all(is_typed_noun);
    let all_values = expressions.iter().all(is_name);
    if all_typed && !typed_allowed {
        reject(
            findings,
            call,
            "typed nouns are not allowed in this control expression",
        );
    } else if !all_typed && !all_values {
        reject(
            findings,
            call,
            "arguments must be either all typed nouns or all noun-value names",
        );
    }
}

fn validate_assignment(findings: &mut Vec<Finding>, assignment: &SyntaxNode) {
    if !has_direct_token(assignment, SyntaxKind::EQ) {
        reject(
            findings,
            assignment,
            "only native `=` assignment is allowed",
        );
        return;
    }

    let children = child_nodes(assignment);
    if children.len() != 2 {
        reject(findings, assignment, "assignment must bind one verb result");
        return;
    }
    let result = &children[0];
    let value = &children[1];

    if !is_name(result) && !is_typed_noun(result) {
        reject(findings, result, "result must be a noun-value name");
    }
    if value.kind() != SyntaxKind::CALL_EXPR {
        reject(findings, value, "assignment value must be a verb call");
        return;
    }

    validate_call(findings, value, true);
    if is_mutating_call(value) {
        reject(
            findings,
            value,
            "call a mutating `verb!` directly instead of assigning its result",
        );
    }
}

fn is_boolean(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::LITERAL
        && (has_direct_token(node, SyntaxKind::TRUE_KW)
            || has_direct_token(node, SyntaxKind::FALSE_KW))
}

fn validate_control_value(findings: &mut Vec<Finding>, value: &SyntaxNode, role: &str) {
    if is_name(value) || is_boolean(value) {
        return;
    }
    if value.kind() == SyntaxKind::CALL_EXPR {
        validate_call(findings, value, false);
        return;
    }
    reject(
        findings,
        value,
        format!("{role} must be a noun-value name, Bool, or verb call"),
    );
}

fn validate_condition(findings: &mut Vec<Finding>, condition: &SyntaxNode) {
    let children = child_nodes(condition);
    if children.len() != 1 {
        reject(findings, condition, "condition must contain one expression");
        return;
    }
    validate_control_value(findings, &children[0], "condition");
}

fn validate_block(findings: &mut Vec<Finding>, block: &SyntaxNode) {
    for statement in block.children() {
        validate_statement(findings, &statement);
    }
}

fn validate_if(findings: &mut Vec<Finding>, expression: &SyntaxNode) {
    for child in expression.children() {
        match child.kind() {
            SyntaxKind::CONDITION => validate_condition(findings, &child),
            SyntaxKind::BLOCK => validate_block(findings, &child),
            SyntaxKind::ELSEIF_CLAUSE => {
                for clause_child in child.children() {
                    match clause_child.kind() {
                        SyntaxKind::CONDITION => validate_condition(findings, &clause_child),
                        SyntaxKind::BLOCK => validate_block(findings, &clause_child),
                        _ => reject(findings, &clause_child, "invalid elseif expression"),
                    }
                }
            }
            SyntaxKind::ELSE_CLAUSE => {
                for clause_child in child.children() {
                    if clause_child.kind() == SyntaxKind::BLOCK {
                        validate_block(findings, &clause_child);
                    } else {
                        reject(findings, &clause_child, "invalid else expression");
                    }
                }
            }
            _ => reject(findings, &child, "invalid if expression"),
        }
    }
}

fn validate_for(findings: &mut Vec<Finding>, expression: &SyntaxNode) {
    for child in expression.children() {
        match child.kind() {
            SyntaxKind::FOR_BINDING => {
                let binding = child_nodes(&child);
                if binding.len() != 2 || !is_name(&binding[0]) {
                    reject(findings, &child, "for must bind one noun-value name");
                } else {
                    validate_control_value(findings, &binding[1], "iteration source");
                }
            }
            SyntaxKind::BLOCK => validate_block(findings, &child),
            _ => reject(findings, &child, "invalid for expression"),
        }
    }
}

fn validate_while(findings: &mut Vec<Finding>, expression: &SyntaxNode) {
    for child in expression.children() {
        match child.kind() {
            SyntaxKind::CONDITION => validate_condition(findings, &child),
            SyntaxKind::BLOCK => validate_block(findings, &child),
            _ => reject(findings, &child, "invalid while expression"),
        }
    }
}

fn validate_subtype(findings: &mut Vec<Finding>, expression: &SyntaxNode) {
    let operands = child_nodes(expression);
    if !has_direct_token(expression, SyntaxKind::SUBTYPE)
        || operands.len() != 2
        || !operands.iter().all(is_type_reference)
    {
        reject(
            findings,
            expression,
            "noun relationship must be `SpecificNoun <: GeneralNoun`",
        );
    }
}

fn validate_statement(findings: &mut Vec<Finding>, statement: &SyntaxNode) {
    match statement.kind() {
        SyntaxKind::BINARY_EXPR => validate_subtype(findings, statement),
        SyntaxKind::CALL_EXPR => validate_call(findings, statement, true),
        SyntaxKind::ASSIGNMENT_EXPR => validate_assignment(findings, statement),
        SyntaxKind::IF_EXPR => validate_if(findings, statement),
        SyntaxKind::FOR_EXPR => validate_for(findings, statement),
        SyntaxKind::WHILE_EXPR => validate_while(findings, statement),
        SyntaxKind::BREAK_EXPR | SyntaxKind::CONTINUE_EXPR => {
            if statement.children().next().is_some() {
                reject(
                    findings,
                    statement,
                    "break and continue cannot carry a value",
                );
            }
        }
        SyntaxKind::BLOCK => validate_block(findings, statement),
        other => reject(
            findings,
            statement,
            format!("{other:?} is outside the idiomatic Julia note subset"),
        ),
    }
}

fn validate_wrapper(findings: &mut Vec<Finding>, root: &SyntaxNode) {
    let items = child_nodes(root);
    if items.len() != 1 || items[0].kind() != SyntaxKind::CONST_STMT {
        reject(
            findings,
            root,
            "file must contain only `const DESIGN = quote ... end`",
        );
        return;
    }

    let declaration = &items[0];
    let assignments: Vec<_> = declaration
        .children()
        .filter(|child| child.kind() == SyntaxKind::ASSIGNMENT_EXPR)
        .collect();
    if assignments.len() != 1 {
        reject(
            findings,
            declaration,
            "DESIGN must have one quoted assignment",
        );
        return;
    }

    let assignment = &assignments[0];
    let children = child_nodes(assignment);
    if children.len() != 2
        || children[0].kind() != SyntaxKind::NAME
        || children[0].text() != "DESIGN"
        || children[1].kind() != SyntaxKind::QUOTE_EXPR
    {
        reject(
            findings,
            assignment,
            "wrapper must be exactly `const DESIGN = quote ... end`",
        );
        return;
    }

    let quote = &children[1];
    let blocks: Vec<_> = quote
        .children()
        .filter(|child| child.kind() == SyntaxKind::BLOCK)
        .collect();
    if blocks.len() != 1 {
        reject(findings, quote, "DESIGN quote must contain one body");
        return;
    }
    validate_block(findings, &blocks[0]);
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let prefix = &source[..offset.min(source.len())];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix, |(_, tail)| tail)
        .chars()
        .count()
        + 1;
    (line, column)
}

fn validate_source(source: &str) -> Vec<Finding> {
    let parsed = parse(source);
    let mut findings = Vec::new();

    for diagnostic in parsed.diagnostics {
        findings.push(Finding {
            offset: diagnostic.start,
            message: format!("Julia parse error: {}", diagnostic.message),
        });
    }
    if findings.is_empty() {
        validate_wrapper(&mut findings, &parsed.cst);
    }

    findings
}

fn check_file(path: &Path) -> Result<bool, String> {
    let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let findings = validate_source(&source);

    if findings.is_empty() {
        println!("{}: valid", path.display());
        return Ok(true);
    }

    for finding in findings {
        let (line, column) = line_column(&source, finding.offset);
        eprintln!("{}:{line}:{column}: {}", path.display(), finding.message);
    }
    Ok(false)
}

fn main() -> ExitCode {
    let paths: Vec<_> = env::args_os().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: idiomatic-julia-check <design.jl>...");
        return ExitCode::from(2);
    }

    let mut valid = true;
    for path in paths {
        let path = Path::new(&path);
        match check_file(path) {
            Ok(result) => valid &= result,
            Err(error) => {
                eprintln!("{}: {error}", path.display());
                valid = false;
            }
        }
    }

    ExitCode::from(if valid { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages(source: &str) -> Vec<String> {
        validate_source(source)
            .into_iter()
            .map(|finding| finding.message)
            .collect()
    }

    #[test]
    fn accepts_the_idiomatic_julia_subset() {
        let source = r#"
const DESIGN = quote
    CsvInput <: Input
    JsonInput <: Input

    output::Output = transform(input::CsvInput, options::Options)
    write!(buffer::Buffer, output::Output)
    transform(input::JsonInput, options::Options)

    output = transform(input, options)
    write!(buffer, output)

    if available(input)
        output = transform(input, options)
    elseif pending(input)
        wait!(input)
    else
        close!(input)
    end

    for record in records(input)
        transform!(record, options)
        if finished(record)
            continue
        end
    end

    while available(input)
        read!(buffer, input)
        if finished(input)
            break
        end
    end
end
"#;

        assert_eq!(messages(source), Vec::<String>::new());
    }

    #[test]
    fn rejects_expressions_outside_the_subset() {
        let cases = [
            ("value = a + b", "assignment value must be a verb call"),
            ("value = verb(other(a), b)", "arguments must be either"),
            ("verb(a.field)", "arguments must be either"),
            (
                "function verb(a)\n a\n end",
                "outside the idiomatic Julia note subset",
            ),
            ("@show a", "outside the idiomatic Julia note subset"),
        ];

        for (body, expected) in cases {
            let source = format!("const DESIGN = quote\n{body}\nend\n");
            let findings = messages(&source);
            assert!(
                findings.iter().any(|message| message.contains(expected)),
                "expected {expected:?} for {body:?}, got {findings:?}"
            );
        }
    }

    #[test]
    fn mutating_calls_cannot_be_assigned() {
        let source = "const DESIGN = quote\nresult = update!(state)\nend\n";
        assert!(
            messages(source)
                .iter()
                .any(|message| message.contains("call a mutating `verb!` directly"))
        );
    }

    #[test]
    fn requires_the_design_quote_wrapper() {
        let source = "result = observe(robot, state)\n";
        assert!(
            messages(source)
                .iter()
                .any(|message| message.contains("const DESIGN = quote"))
        );
    }
}
