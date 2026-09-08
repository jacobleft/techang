mod semantic;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
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

fn is_name_or_dot_access(node: &SyntaxNode) -> bool {
    if is_name(node) {
        return true;
    }
    if node.kind() != SyntaxKind::BINARY_EXPR || !has_direct_token(node, SyntaxKind::DOT) {
        return false;
    }
    let parts = child_nodes(node);
    parts.len() == 2 && is_name_or_dot_access(&parts[0]) && is_name(&parts[1])
}

fn terminal_segment(node: &SyntaxNode) -> Option<String> {
    if is_name(node) {
        return Some(node.text().to_string());
    }
    is_name_or_dot_access(node)
        .then(|| child_nodes(node).last().and_then(terminal_segment))
        .flatten()
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
            if is_name_or_dot_access(node) {
                return true;
            }
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
    children.len() == 2 && is_name_or_dot_access(&children[0]) && is_type_reference(&children[1])
}

fn is_typed_binding(node: &SyntaxNode) -> bool {
    if !is_typed_noun(node) {
        return false;
    }
    child_nodes(node).first().is_some_and(is_name)
}

fn is_simple_value(node: &SyntaxNode) -> bool {
    is_name_or_dot_access(node)
        || matches!(node.kind(), SyntaxKind::LITERAL | SyntaxKind::QUOTE_SYM)
}

fn callable_name(call: &SyntaxNode) -> Option<String> {
    let callee = child_nodes(call).into_iter().next()?;
    if !is_name_or_dot_access(&callee) {
        return None;
    }
    terminal_segment(&callee)
}

fn is_mutating_call(call: &SyntaxNode) -> bool {
    callable_name(call).is_some_and(|name| name.ends_with('!'))
}

fn is_noun_argument(expression: &SyntaxNode, typed_allowed: bool) -> bool {
    is_name_or_dot_access(expression) || (typed_allowed && is_typed_noun(expression))
}

fn is_keyword_argument(argument: &SyntaxNode, typed_allowed: bool) -> bool {
    match argument.kind() {
        SyntaxKind::ARG => {
            let expressions = child_nodes(argument);
            if expressions.len() == 1 && expressions[0].kind() == SyntaxKind::ASSIGNMENT_EXPR {
                let assignment = &expressions[0];
                let parts = child_nodes(assignment);
                has_direct_token(assignment, SyntaxKind::EQ)
                    && parts.len() == 2
                    && (is_name(&parts[0]) || (typed_allowed && is_typed_binding(&parts[0])))
                    && is_simple_value(&parts[1])
            } else {
                expressions.len() == 1 && is_noun_argument(&expressions[0], typed_allowed)
            }
        }
        SyntaxKind::KEYWORD_ARG => {
            let children = child_nodes(argument);
            children.len() == 2 && is_name(&children[0]) && is_simple_value(&children[1])
        }
        _ => false,
    }
}

fn validate_call(findings: &mut Vec<Finding>, call: &SyntaxNode, typed_allowed: bool) {
    if callable_name(call).is_none() {
        reject(findings, call, "call must use a name or dotted callable");
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

    let mut argument_count = 0;
    for argument in arguments.children() {
        match argument.kind() {
            SyntaxKind::ARG => {
                argument_count += 1;
                let expressions = child_nodes(&argument);
                if expressions.len() != 1 || !is_noun_argument(&expressions[0], typed_allowed) {
                    reject(
                        findings,
                        &argument,
                        "argument must be a noun value or typed noun",
                    );
                }
            }
            SyntaxKind::PARAMETERS => {
                for keyword in argument.children() {
                    argument_count += 1;
                    if !is_keyword_argument(&keyword, typed_allowed) {
                        reject(
                            findings,
                            &keyword,
                            "keyword argument must use a noun value, literal, or typed default",
                        );
                    }
                }
            }
            _ => reject(findings, &argument, "invalid call argument"),
        }
    }
    if argument_count == 0 {
        reject(findings, call, "verb must act on at least one noun");
    }
}

fn annotated_call(annotation: &SyntaxNode) -> Option<(SyntaxNode, SyntaxNode)> {
    if annotation.kind() != SyntaxKind::TYPE_ANNOTATION {
        return None;
    }
    let children = child_nodes(annotation);
    (children.len() == 2 && children[0].kind() == SyntaxKind::CALL_EXPR)
        .then(|| (children[0].clone(), children[1].clone()))
}

fn validate_mutation_assignment(findings: &mut Vec<Finding>, assignment: &SyntaxNode) {
    let children = child_nodes(assignment);
    if children.len() != 2 {
        reject(
            findings,
            assignment,
            "mutation assignment must have a target and value",
        );
        return;
    }
    if !is_name_or_dot_access(&children[0]) {
        reject(
            findings,
            &children[0],
            "mutation target must be a noun value or owned field",
        );
    }
    let value = &children[1];
    if value.kind() == SyntaxKind::CALL_EXPR {
        validate_call(findings, value, false);
    } else if !is_simple_value(value) {
        reject(
            findings,
            value,
            "mutation value must be a simple noun value, literal, or verb call",
        );
    }
}

fn validate_result_assignment(findings: &mut Vec<Finding>, assignment: &SyntaxNode) {
    if !has_direct_token(assignment, SyntaxKind::EQ) {
        reject(
            findings,
            assignment,
            "assignment must use `=`, `.=` or `+=`",
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

    if !is_name(result) && !is_typed_binding(result) {
        reject(findings, result, "result must be a noun-value name");
    }
    let (call, return_type) = if value.kind() == SyntaxKind::CALL_EXPR {
        (value.clone(), None)
    } else if let Some((call, return_type)) = annotated_call(value) {
        (call, Some(return_type))
    } else {
        reject(findings, value, "assignment value must be a verb call");
        return;
    };

    if return_type.is_some_and(|return_type| !is_type_reference(&return_type)) {
        reject(findings, value, "return annotation must name a noun type");
    }
    validate_call(findings, &call, true);
    if is_mutating_call(&call) {
        reject(
            findings,
            &call,
            "call a mutating `verb!` directly instead of assigning its result",
        );
    }
}

fn validate_assignment(findings: &mut Vec<Finding>, assignment: &SyntaxNode) {
    if has_direct_token(assignment, SyntaxKind::DOT_EQ)
        || has_direct_token(assignment, SyntaxKind::PLUS_EQ)
    {
        validate_mutation_assignment(findings, assignment);
    } else {
        validate_result_assignment(findings, assignment);
    }
}

fn is_boolean(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::LITERAL
        && (has_direct_token(node, SyntaxKind::TRUE_KW)
            || has_direct_token(node, SyntaxKind::FALSE_KW))
}

fn validate_control_value(findings: &mut Vec<Finding>, value: &SyntaxNode, role: &str) {
    if is_name_or_dot_access(value) || is_boolean(value) {
        return;
    }
    if value.kind() == SyntaxKind::CALL_EXPR {
        validate_call(findings, value, false);
        return;
    }
    reject(
        findings,
        value,
        format!("{role} must be a noun value, Bool, or verb call"),
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

fn is_binding_pattern(node: &SyntaxNode) -> bool {
    if is_name(node) {
        return true;
    }
    if node.kind() != SyntaxKind::TUPLE_EXPR {
        return false;
    }
    let elements: Vec<_> = node.children().collect();
    !elements.is_empty()
        && elements.iter().all(|element| {
            element.kind() == SyntaxKind::ARG
                && child_nodes(element).first().is_some_and(is_binding_pattern)
        })
}

fn validate_for(findings: &mut Vec<Finding>, expression: &SyntaxNode) {
    for child in expression.children() {
        match child.kind() {
            SyntaxKind::FOR_BINDING => {
                let binding = child_nodes(&child);
                if binding.len() != 2 || !is_binding_pattern(&binding[0]) {
                    reject(
                        findings,
                        &child,
                        "for must bind a noun name or tuple of noun names",
                    );
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

fn validate_type_annotation(findings: &mut Vec<Finding>, annotation: &SyntaxNode) {
    let children = child_nodes(annotation);
    if children.len() != 2 || !is_type_reference(&children[1]) {
        reject(
            findings,
            annotation,
            "type annotation must name a noun type",
        );
        return;
    }
    if children[0].kind() == SyntaxKind::CALL_EXPR {
        validate_call(findings, &children[0], true);
    } else if children[0].kind() == SyntaxKind::BINARY_EXPR && is_name_or_dot_access(&children[0]) {
        // `owner.field::FieldNoun` records which object owns a field.
    } else {
        reject(
            findings,
            &children[0],
            "type annotation must describe a verb return or owned field",
        );
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
        SyntaxKind::BINARY_EXPR if is_name_or_dot_access(statement) => {}
        SyntaxKind::BINARY_EXPR => validate_subtype(findings, statement),
        SyntaxKind::CALL_EXPR => validate_call(findings, statement, true),
        SyntaxKind::TYPE_ANNOTATION => validate_type_annotation(findings, statement),
        SyntaxKind::ASSIGNMENT_EXPR => validate_assignment(findings, statement),
        SyntaxKind::FUNCTION_DEF => {}
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

fn validate_quote(findings: &mut Vec<Finding>, quote: &SyntaxNode) {
    let blocks: Vec<_> = quote
        .children()
        .filter(|child| child.kind() == SyntaxKind::BLOCK)
        .collect();
    if blocks.len() != 1 {
        reject(findings, quote, "quote must contain one body");
        return;
    }
    validate_block(findings, &blocks[0]);
}

fn validate_const_quote(findings: &mut Vec<Finding>, declaration: &SyntaxNode) {
    let assignments: Vec<_> = declaration
        .children()
        .filter(|child| child.kind() == SyntaxKind::ASSIGNMENT_EXPR)
        .collect();
    if assignments.len() != 1 {
        reject(findings, declaration, "const label must assign one quote");
        return;
    }

    let assignment = &assignments[0];
    let children = child_nodes(assignment);
    if children.len() != 2
        || children[0].kind() != SyntaxKind::NAME
        || children[1].kind() != SyntaxKind::QUOTE_EXPR
    {
        reject(
            findings,
            assignment,
            "top-level const must be `const NAME = quote ... end`",
        );
        return;
    }
    validate_quote(findings, &children[1]);
}

fn validate_file(findings: &mut Vec<Finding>, root: &SyntaxNode) {
    let items = child_nodes(root);
    if items.is_empty() {
        reject(findings, root, "file must contain at least one quote");
        return;
    }

    for item in items {
        match item.kind() {
            SyntaxKind::QUOTE_EXPR => validate_quote(findings, &item),
            SyntaxKind::CONST_STMT => validate_const_quote(findings, &item),
            _ => reject(
                findings,
                &item,
                "top-level item must be `quote ... end` or `const NAME = quote ... end`",
            ),
        }
    }
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
        validate_file(&mut findings, &parsed.cst);
    }

    findings
}

fn check_file(path: &Path, semantic_root: Option<&Path>) -> Result<bool, String> {
    let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let findings = validate_source(&source);

    if findings.is_empty() {
        println!("{}: valid", path.display());
        let Some(semantic_root) = semantic_root else {
            return Ok(true);
        };
        let semantic_findings = semantic::check_semantics(semantic_root, &source)?;
        if semantic_findings.is_empty() {
            println!("{}: semantic: no design signatures", path.display());
            return Ok(true);
        }
        let mut compatible = true;
        for finding in semantic_findings {
            let (line, column) = line_column(&source, finding.offset);
            println!(
                "{}:{line}:{column}: semantic {}: {} — {}",
                path.display(),
                finding.status.label(),
                finding.signature,
                finding.detail
            );
            compatible &= finding.status.is_compatible();
        }
        return Ok(compatible);
    }

    for finding in findings {
        let (line, column) = line_column(&source, finding.offset);
        eprintln!("{}:{line}:{column}: {}", path.display(), finding.message);
    }
    Ok(false)
}

fn parse_args() -> Result<(Option<PathBuf>, Vec<OsString>), String> {
    let mut arguments = env::args_os().skip(1);
    let mut semantic_root = None;
    let mut paths = Vec::new();
    while let Some(argument) = arguments.next() {
        if argument == "--semantic" {
            let Some(root) = arguments.next() else {
                return Err("`--semantic` requires a package root".to_string());
            };
            semantic_root = Some(PathBuf::from(root));
        } else if argument.to_string_lossy().starts_with('-') {
            return Err(format!("unknown option `{}`", argument.to_string_lossy()));
        } else {
            paths.push(argument);
        }
    }
    Ok((semantic_root, paths))
}

fn main() -> ExitCode {
    let (semantic_root, paths) = match parse_args() {
        Ok(arguments) => arguments,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    if paths.is_empty() {
        eprintln!("usage: idiomatic-julia-check [--semantic <package-root>] <file>...");
        return ExitCode::from(2);
    }

    let mut valid = true;
    for path in paths {
        let path = Path::new(&path);
        match check_file(path, semantic_root.as_deref()) {
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
    fn accepts_multiple_quotes_and_comments() {
        let source = r#"
# Noun relationships
const NOUNS = quote
    CsvInput <: Input # line comment inside a quote
end

#=
Operation declarations can have their own quoted block.
=#
const OPERATIONS = quote
    output::Output = transform(input::Input, options::Options)
    write!(buffer::Buffer, output::Output)
end

# Bare quotes are also valid.
quote
    output = transform(input, options)
    write!(buffer, output)
end
"#;

        assert_eq!(messages(source), Vec::<String>::new());
    }

    #[test]
    fn accepts_dot_access_and_algorithm_body() {
        let source = r#"
const CORE = quote
    PackageName.ConcreteNoun(items)
    PackageName.AbstractNoun
    PackageName.verb!(object, state)
    combine!(destination, workspace.intermediate, item.data)
    result::PackageName.ResultNoun = PackageName.verb(input::PackageName.InputNoun)

    if state.ready
        PackageName.verb!(object.state, context.cache.value)
    end
end

const ALGORITHM = quote
    function transform!(destination, workspace, item::ConcreteNoun{Variant})
        combine!(
            destination,
            workspace.intermediate,
            workspace.parameters,
            item.data,
        )
    end
end
"#;

        assert_eq!(messages(source), Vec::<String>::new());
    }

    #[test]
    fn accepts_extended_surface_notation() {
        let source = r#"
const SURFACE = quote
    verb(a::NounA, b; option::OptionNoun = default)::ResultNoun
    result = verb(a::NounA, b; option = settings.option, mode = :fast)::ResultNoun
    owner.field::FieldNoun

    for (key, value) in pairs(source)
        update!(destination, key, value; mode = settings.mode)
    end

    destination .= source.values
    owner.field += increment
end
"#;

        assert_eq!(messages(source), Vec::<String>::new());
    }

    #[test]
    fn rejects_expressions_outside_the_subset() {
        let cases = [
            ("value = a + b", "assignment value must be a verb call"),
            ("value = verb(other(a), b)", "argument must be"),
            ("verb(a[1])", "argument must be"),
            (
                "verb(a; option = other(b))",
                "keyword argument must use a noun value",
            ),
            (
                "struct Container\n value\n end",
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
        for call in ["update!(state)", "PackageName.update!(state)"] {
            let source = format!("const DESIGN = quote\nresult = {call}\nend\n");
            assert!(
                messages(&source)
                    .iter()
                    .any(|message| message.contains("call a mutating `verb!` directly")),
                "expected assigned mutating call to fail: {call}"
            );
        }
    }

    #[test]
    fn rejects_other_assignment_operators() {
        let source = "const DESIGN = quote\ntarget *= value\nend\n";
        assert!(
            messages(source)
                .iter()
                .any(|message| message.contains("assignment must use `=`, `.=` or `+=`"))
        );
    }

    #[test]
    fn requires_quoted_top_level_items() {
        let source = "result = observe(robot, state)\n";
        assert!(
            messages(source)
                .iter()
                .any(|message| message.contains("top-level item must be"))
        );
    }

    #[test]
    fn comments_alone_are_not_a_design() {
        let source = "# no quoted design yet\n#= still no design =#\n";
        assert!(
            messages(source)
                .iter()
                .any(|message| message.contains("at least one quote"))
        );
    }
}
