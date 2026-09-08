use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use fatou_parser::parser::parse;
use fatou_parser::syntax::{SyntaxKind, SyntaxNode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApiStatus {
    Planned,
    Exact,
    Covered,
    Missing,
    Unknown,
}

impl ApiStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Exact => "exact",
            Self::Covered => "covered",
            Self::Missing => "missing",
            Self::Unknown => "unknown",
        }
    }

    pub fn is_compatible(self) -> bool {
        matches!(self, Self::Planned | Self::Exact | Self::Covered)
    }
}

#[derive(Debug)]
pub struct ApiFinding {
    pub offset: usize,
    pub status: ApiStatus,
    pub signature: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TypeSpec {
    text: String,
    known: bool,
}

impl TypeSpec {
    fn any() -> Self {
        Self {
            text: "Any".to_string(),
            known: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct KeywordSpec {
    name: String,
    value_type: TypeSpec,
    has_default: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SignatureOrigin {
    Declaration,
    Requirement,
    Implementation,
}

#[derive(Clone, Debug)]
struct MethodSignature {
    callable: String,
    qualifier: Option<String>,
    owner: Option<String>,
    origin: SignatureOrigin,
    positional: Vec<TypeSpec>,
    keywords: Vec<KeywordSpec>,
    return_type: Option<TypeSpec>,
    vararg: bool,
    uncertain: bool,
    offset: usize,
}

impl MethodSignature {
    fn display(&self) -> String {
        let mut arguments: Vec<_> = self
            .positional
            .iter()
            .map(|argument| format!("::{}", argument.text))
            .collect();
        if self.vararg
            && let Some(last) = arguments.last_mut()
        {
            last.push_str("...");
        }
        let callable = self.qualifier.as_ref().map_or_else(
            || self.callable.clone(),
            |qualifier| format!("{qualifier}.{}", self.callable),
        );
        let mut display = format!("{callable}({}", arguments.join(", "));
        if !self.keywords.is_empty() {
            let keywords: Vec<_> = self
                .keywords
                .iter()
                .map(|keyword| {
                    format!(
                        "{}::{}{}",
                        keyword.name,
                        keyword.value_type.text,
                        if keyword.has_default { " = …" } else { "" }
                    )
                })
                .collect();
            display.push_str(&format!("; {}", keywords.join(", ")));
        }
        display.push(')');
        if let Some(return_type) = &self.return_type {
            display.push_str(&format!("::{}", return_type.text));
        }
        display
    }
}

#[derive(Default)]
struct ApiIndex {
    methods: Vec<MethodSignature>,
    parents: HashMap<String, String>,
    types: HashSet<String>,
    bindings: HashSet<(String, String)>,
    dependencies: HashSet<String>,
    unresolved_modules: HashMap<String, String>,
}

fn children(node: &SyntaxNode) -> Vec<SyntaxNode> {
    node.children().collect()
}

fn has_token(node: &SyntaxNode, kind: SyntaxKind) -> bool {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .any(|token| token.kind() == kind)
}

fn compact_text(node: &SyntaxNode) -> String {
    node.text()
        .to_string()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn path_segments(node: &SyntaxNode) -> Option<Vec<String>> {
    match node.kind() {
        SyntaxKind::NAME => Some(vec![node.text().to_string()]),
        SyntaxKind::BINARY_EXPR if has_token(node, SyntaxKind::DOT) => {
            let parts = children(node);
            if parts.len() != 2 || parts[1].kind() != SyntaxKind::NAME {
                return None;
            }
            let mut segments = path_segments(&parts[0])?;
            segments.push(parts[1].text().to_string());
            Some(segments)
        }
        _ => None,
    }
}

fn type_spec(node: &SyntaxNode) -> TypeSpec {
    match node.kind() {
        SyntaxKind::NAME => TypeSpec {
            text: node.text().to_string(),
            known: true,
        },
        SyntaxKind::BINARY_EXPR => {
            if let Some(segments) = path_segments(node) {
                return TypeSpec {
                    text: segments.last().cloned().unwrap_or_default(),
                    known: true,
                };
            }
            TypeSpec {
                text: compact_text(node),
                known: false,
            }
        }
        SyntaxKind::CURLY_EXPR => {
            let parts = children(node);
            if parts.len() != 2 || parts[1].kind() != SyntaxKind::ARG_LIST {
                return TypeSpec {
                    text: compact_text(node),
                    known: false,
                };
            }
            let base = type_spec(&parts[0]);
            let arguments: Vec<_> = parts[1]
                .children()
                .filter(|argument| argument.kind() == SyntaxKind::ARG)
                .filter_map(|argument| children(&argument).first().cloned())
                .map(|argument| type_spec(&argument))
                .collect();
            TypeSpec {
                text: format!(
                    "{}{{{}}}",
                    base.text,
                    arguments
                        .iter()
                        .map(|argument| argument.text.as_str())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                known: base.known && arguments.iter().all(|argument| argument.known),
            }
        }
        _ => TypeSpec {
            text: compact_text(node),
            known: false,
        },
    }
}

fn typed_parameter(node: &SyntaxNode) -> Option<(String, TypeSpec)> {
    if node.kind() != SyntaxKind::TYPE_ANNOTATION {
        return None;
    }
    let parts = children(node);
    if parts.len() != 2 || parts[0].kind() != SyntaxKind::NAME {
        return None;
    }
    Some((parts[0].text().to_string(), type_spec(&parts[1])))
}

fn positional_parameter(node: &SyntaxNode) -> (TypeSpec, bool, bool) {
    if let Some((_, value_type)) = typed_parameter(node) {
        let vararg = value_type.text.starts_with("Vararg{");
        return (value_type, vararg, false);
    }
    if node.kind() == SyntaxKind::SPLAT_EXPR {
        let parts = children(node);
        if parts.len() == 1 {
            let (value_type, _, uncertain) = positional_parameter(&parts[0]);
            return (value_type, true, uncertain);
        }
        return (TypeSpec::any(), true, true);
    }
    if node.kind() == SyntaxKind::NAME {
        return (TypeSpec::any(), false, false);
    }
    (TypeSpec::any(), false, true)
}

fn keyword_parameter(node: &SyntaxNode) -> Option<KeywordSpec> {
    let expression = children(node).first()?.clone();
    if node.kind() == SyntaxKind::KEYWORD_ARG {
        let parts = children(node);
        return (parts.len() == 2 && parts[0].kind() == SyntaxKind::NAME).then(|| KeywordSpec {
            name: parts[0].text().to_string(),
            value_type: TypeSpec::any(),
            has_default: true,
        });
    }
    if expression.kind() == SyntaxKind::ASSIGNMENT_EXPR {
        let parts = children(&expression);
        if parts.len() != 2 {
            return None;
        }
        if let Some((name, value_type)) = typed_parameter(&parts[0]) {
            return Some(KeywordSpec {
                name,
                value_type,
                has_default: true,
            });
        }
        return (parts[0].kind() == SyntaxKind::NAME).then(|| KeywordSpec {
            name: parts[0].text().to_string(),
            value_type: TypeSpec::any(),
            has_default: true,
        });
    }
    if let Some((name, value_type)) = typed_parameter(&expression) {
        return Some(KeywordSpec {
            name,
            value_type,
            has_default: false,
        });
    }
    (expression.kind() == SyntaxKind::NAME).then(|| KeywordSpec {
        name: expression.text().to_string(),
        value_type: TypeSpec::any(),
        has_default: false,
    })
}

fn signature_expression(node: &SyntaxNode) -> Option<(SyntaxNode, Option<TypeSpec>, bool)> {
    match node.kind() {
        SyntaxKind::SIGNATURE => {
            let expression = children(node).first()?.clone();
            signature_expression(&expression)
        }
        SyntaxKind::WHERE_EXPR => {
            let expression = children(node).first()?.clone();
            let (call, return_type, _) = signature_expression(&expression)?;
            Some((call, return_type, true))
        }
        SyntaxKind::TYPE_ANNOTATION => {
            let parts = children(node);
            (parts.len() == 2 && parts[0].kind() == SyntaxKind::CALL_EXPR)
                .then(|| (parts[0].clone(), Some(type_spec(&parts[1])), false))
        }
        SyntaxKind::CALL_EXPR => Some((node.clone(), None, false)),
        _ => None,
    }
}

fn method_signature(node: &SyntaxNode) -> Option<MethodSignature> {
    let (call, return_type, uncertain) = signature_expression(node)?;
    let call_parts = children(&call);
    let callee = call_parts.first()?;
    let segments = path_segments(callee)?;
    let callable = segments.last()?.clone();
    let qualifier = (segments.len() > 1).then(|| segments[..segments.len() - 1].join("."));
    let arguments = call_parts
        .iter()
        .find(|part| part.kind() == SyntaxKind::ARG_LIST)?;
    let mut positional = Vec::new();
    let mut keywords = Vec::new();
    let mut vararg = false;
    let mut uncertain = uncertain;
    for argument in arguments.children() {
        match argument.kind() {
            SyntaxKind::ARG => {
                let expression = children(&argument).first()?.clone();
                let (value_type, argument_vararg, argument_uncertain) =
                    positional_parameter(&expression);
                positional.push(value_type);
                vararg |= argument_vararg;
                uncertain |= argument_uncertain;
            }
            SyntaxKind::PARAMETERS => {
                for keyword in argument.children() {
                    if let Some(keyword) = keyword_parameter(&keyword) {
                        keywords.push(keyword);
                    } else {
                        uncertain = true;
                    }
                }
            }
            _ => uncertain = true,
        }
    }
    Some(MethodSignature {
        callable,
        qualifier,
        owner: None,
        origin: SignatureOrigin::Declaration,
        positional,
        keywords,
        return_type,
        vararg,
        uncertain,
        offset: u32::from(call.text_range().start()) as usize,
    })
}

fn return_type_from_binding(node: &SyntaxNode) -> Option<TypeSpec> {
    let parts = children(node);
    (node.kind() == SyntaxKind::TYPE_ANNOTATION && parts.len() == 2).then(|| type_spec(&parts[1]))
}

fn signature_has_types(signature: &MethodSignature) -> bool {
    signature.return_type.is_some()
        || signature
            .positional
            .iter()
            .any(|argument| argument.text != "Any")
        || signature
            .keywords
            .iter()
            .any(|keyword| keyword.value_type.text != "Any")
}

fn same_node(left: &SyntaxNode, right: &SyntaxNode) -> bool {
    left.kind() == right.kind() && left.text_range() == right.text_range()
}

fn inside_function(node: &SyntaxNode) -> bool {
    node.ancestors()
        .skip(1)
        .any(|ancestor| ancestor.kind() == SyntaxKind::FUNCTION_DEF)
}

fn has_ancestor(node: &SyntaxNode, kind: SyntaxKind) -> bool {
    node.ancestors()
        .skip(1)
        .any(|ancestor| ancestor.kind() == kind)
}

fn design_signatures(root: &SyntaxNode) -> Vec<MethodSignature> {
    let mut signatures = Vec::new();
    for quote in root
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::QUOTE_EXPR)
    {
        let Some(block) = quote
            .children()
            .find(|child| child.kind() == SyntaxKind::BLOCK)
        else {
            continue;
        };
        for node in block.descendants().skip(1) {
            if inside_function(&node) {
                continue;
            }
            let top_level = node
                .parent()
                .is_some_and(|parent| same_node(&parent, &block));
            let signature = match node.kind() {
                SyntaxKind::FUNCTION_DEF => node
                    .children()
                    .find(|child| child.kind() == SyntaxKind::SIGNATURE)
                    .and_then(|signature| method_signature(&signature))
                    .map(|mut signature| {
                        signature.origin = SignatureOrigin::Declaration;
                        signature
                    }),
                SyntaxKind::TYPE_ANNOTATION
                    if !has_ancestor(&node, SyntaxKind::ASSIGNMENT_EXPR) =>
                {
                    method_signature(&node).map(|mut signature| {
                        signature.origin = if top_level && signature_has_types(&signature) {
                            SignatureOrigin::Declaration
                        } else {
                            SignatureOrigin::Requirement
                        };
                        signature
                    })
                }
                SyntaxKind::CALL_EXPR
                    if !has_ancestor(&node, SyntaxKind::ASSIGNMENT_EXPR)
                        && !has_ancestor(&node, SyntaxKind::TYPE_ANNOTATION)
                        && !has_ancestor(&node, SyntaxKind::CALL_EXPR) =>
                {
                    method_signature(&node).map(|mut signature| {
                        signature.origin = if top_level && signature_has_types(&signature) {
                            SignatureOrigin::Declaration
                        } else {
                            SignatureOrigin::Requirement
                        };
                        signature
                    })
                }
                SyntaxKind::ASSIGNMENT_EXPR if has_token(&node, SyntaxKind::EQ) => {
                    let parts = children(&node);
                    if parts.len() != 2 {
                        None
                    } else {
                        method_signature(&parts[1]).map(|mut signature| {
                            signature.origin = SignatureOrigin::Requirement;
                            if signature.return_type.is_none() {
                                signature.return_type = return_type_from_binding(&parts[0]);
                            }
                            signature
                        })
                    }
                }
                _ => None,
            };
            if let Some(signature) = signature {
                signatures.push(signature);
            }
        }
    }
    signatures.sort_by_key(|signature| signature.offset);
    signatures
}

fn is_nested_method(node: &SyntaxNode) -> bool {
    node.ancestors()
        .skip(1)
        .any(|ancestor| ancestor.kind() == SyntaxKind::FUNCTION_DEF)
}

fn definition_is_dynamic(node: &SyntaxNode) -> bool {
    node.ancestors().skip(1).any(|ancestor| {
        matches!(
            ancestor.kind(),
            SyntaxKind::MACRO_CALL | SyntaxKind::QUOTE_EXPR | SyntaxKind::IF_EXPR
        )
    })
}

fn declaration_name(node: &SyntaxNode) -> Option<String> {
    match node.kind() {
        SyntaxKind::NAME => Some(node.text().to_string()),
        SyntaxKind::CURLY_EXPR => children(node).first().and_then(declaration_name),
        _ => path_segments(node).and_then(|segments| segments.last().cloned()),
    }
}

fn index_type(index: &mut ApiIndex, declaration: &SyntaxNode, owner: &str) {
    let Some(signature) = declaration
        .children()
        .find(|child| child.kind() == SyntaxKind::SIGNATURE)
    else {
        return;
    };
    let Some(expression) = children(&signature).first().cloned() else {
        return;
    };
    let parts = children(&expression);
    let (type_node, parent) = if expression.kind() == SyntaxKind::BINARY_EXPR
        && has_token(&expression, SyntaxKind::SUBTYPE)
        && parts.len() == 2
    {
        (&parts[0], Some(type_spec(&parts[1]).text))
    } else {
        (&expression, None)
    };
    if let Some(name) = declaration_name(type_node) {
        if let Some(parent) = parent {
            index.parents.insert(name.clone(), parent);
        }
        index.bindings.insert((owner.to_string(), name.clone()));
        index.types.insert(name);
    }
}

fn signature_owner(signature: &MethodSignature, default_owner: &str) -> String {
    signature
        .qualifier
        .as_deref()
        .and_then(|qualifier| qualifier.split('.').next())
        .unwrap_or(default_owner)
        .to_string()
}

fn index_method(index: &mut ApiIndex, mut signature: MethodSignature, owner: &str) {
    signature.origin = SignatureOrigin::Implementation;
    signature.owner = Some(signature_owner(&signature, owner));
    index.bindings.insert((
        signature.owner.clone().unwrap_or_else(|| owner.to_string()),
        signature.callable.clone(),
    ));
    index.methods.push(signature);
}

fn index_source_files(
    index: &mut ApiIndex,
    source_files: &[(PathBuf, String)],
    owner: &str,
) -> Result<(), String> {
    for (path, source) in source_files {
        let parsed = parse(source);
        if let Some(diagnostic) = parsed.diagnostics.first() {
            return Err(format!(
                "{}: Julia parse error: {}",
                path.display(),
                diagnostic.message
            ));
        }
        for node in parsed.cst.descendants() {
            match node.kind() {
                SyntaxKind::FUNCTION_DEF if !is_nested_method(&node) => {
                    let signature_node = node
                        .children()
                        .find(|child| child.kind() == SyntaxKind::SIGNATURE);
                    if let Some(mut signature) = signature_node.as_ref().and_then(method_signature)
                    {
                        signature.uncertain |= definition_is_dynamic(&node);
                        index_method(index, signature, owner);
                    } else if let Some(name) = signature_node
                        .and_then(|signature| children(&signature).first().cloned())
                        .and_then(|expression| declaration_name(&expression))
                    {
                        index.bindings.insert((owner.to_string(), name));
                    }
                }
                SyntaxKind::ASSIGNMENT_EXPR
                    if has_token(&node, SyntaxKind::EQ) && !is_nested_method(&node) =>
                {
                    if let Some(left) = children(&node).first()
                        && let Some(mut signature) = method_signature(left)
                    {
                        signature.uncertain |= definition_is_dynamic(&node);
                        index_method(index, signature, owner);
                    }
                }
                SyntaxKind::STRUCT_DEF | SyntaxKind::ABSTRACT_DEF => {
                    index_type(index, &node, owner);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

#[cfg(test)]
fn source_index(source_files: &[(PathBuf, String)], owner: &str) -> Result<ApiIndex, String> {
    let mut index = ApiIndex::default();
    index_source_files(&mut index, source_files, owner)?;
    Ok(index)
}

fn base_type(value_type: &TypeSpec) -> &str {
    value_type
        .text
        .split_once('{')
        .map_or(value_type.text.as_str(), |(base, _)| base)
}

fn subtype_relation(required: &TypeSpec, provided: &TypeSpec, index: &ApiIndex) -> Option<bool> {
    if !required.known || !provided.known {
        return None;
    }
    if required.text == provided.text || provided.text == "Any" {
        return Some(true);
    }
    let target = base_type(provided);
    let mut current = base_type(required);
    let mut visited = HashSet::new();
    while visited.insert(current.to_string()) {
        let Some(parent) = index.parents.get(current) else {
            return Some(false);
        };
        if base_type(&TypeSpec {
            text: parent.clone(),
            known: true,
        }) == target
        {
            return Some(true);
        }
        current = parent;
    }
    Some(false)
}

fn signature_coverage(
    required: &MethodSignature,
    provided: &MethodSignature,
    index: &ApiIndex,
) -> Option<bool> {
    if required.vararg || provided.vararg || required.uncertain || provided.uncertain {
        return None;
    }
    if required.positional.len() != provided.positional.len() {
        return Some(false);
    }
    for (required_type, provided_type) in required.positional.iter().zip(&provided.positional) {
        match subtype_relation(required_type, provided_type, index) {
            Some(true) => {}
            other => return other,
        }
    }
    for required_keyword in &required.keywords {
        let Some(provided_keyword) = provided
            .keywords
            .iter()
            .find(|keyword| keyword.name == required_keyword.name)
        else {
            return Some(false);
        };
        match subtype_relation(
            &required_keyword.value_type,
            &provided_keyword.value_type,
            index,
        ) {
            Some(true) => {}
            other => return other,
        }
    }
    if provided.keywords.iter().any(|keyword| {
        !keyword.has_default && !required.keywords.iter().any(|r| r.name == keyword.name)
    }) {
        return Some(false);
    }
    if let Some(required_return) = &required.return_type {
        let provided_return = provided.return_type.as_ref()?;
        return subtype_relation(provided_return, required_return, index);
    }
    Some(true)
}

fn signatures_equal(required: &MethodSignature, provided: &MethodSignature) -> bool {
    required.positional == provided.positional
        && required.keywords == provided.keywords
        && required.return_type == provided.return_type
        && required.vararg == provided.vararg
        && !required.uncertain
        && !provided.uncertain
}

fn compare_signature(
    required: &MethodSignature,
    index: &ApiIndex,
    package_name: Option<&str>,
) -> ApiFinding {
    let display = required.display();
    let qualified_owner = required
        .qualifier
        .as_deref()
        .and_then(|qualifier| qualifier.split('.').next());
    let target_owner = qualified_owner.or(package_name);
    if required
        .qualifier
        .as_deref()
        .is_some_and(|qualifier| qualifier.contains('.'))
    {
        return ApiFinding {
            offset: required.offset,
            status: ApiStatus::Unknown,
            signature: display,
            detail: "nested module ownership cannot be resolved statically".to_string(),
        };
    }
    if let Some(owner) = qualified_owner
        && package_name != Some(owner)
        && !index.dependencies.contains(owner)
        && !index
            .methods
            .iter()
            .any(|method| method.owner.as_deref() == Some(owner))
    {
        return ApiFinding {
            offset: required.offset,
            status: ApiStatus::Missing,
            signature: display,
            detail: format!("{owner} is not a direct Manifest-bound dependency"),
        };
    }
    if let Some(owner) = target_owner
        && let Some(reason) = index.unresolved_modules.get(owner)
    {
        return ApiFinding {
            offset: required.offset,
            status: ApiStatus::Unknown,
            signature: display,
            detail: reason.clone(),
        };
    }
    let candidates: Vec<_> = index
        .methods
        .iter()
        .filter(|method| {
            method.callable == required.callable
                && target_owner.is_none_or(|owner| method.owner.as_deref() == Some(owner))
        })
        .collect();
    let value_only = required.return_type.is_none()
        && required
            .positional
            .iter()
            .all(|argument| argument.text == "Any")
        && required
            .keywords
            .iter()
            .all(|keyword| keyword.value_type.text == "Any");
    if value_only
        && candidates.iter().any(|candidate| {
            candidate.positional.len() == required.positional.len()
                && required.keywords.iter().all(|required_keyword| {
                    candidate
                        .keywords
                        .iter()
                        .any(|keyword| keyword.name == required_keyword.name)
                })
        })
    {
        return ApiFinding {
            offset: required.offset,
            status: ApiStatus::Covered,
            signature: display,
            detail: "planned callable and argument shape found; value types are unspecified"
                .to_string(),
        };
    }
    if candidates
        .iter()
        .any(|candidate| signatures_equal(required, candidate))
    {
        return ApiFinding {
            offset: required.offset,
            status: ApiStatus::Exact,
            signature: display,
            detail: "matching method signature found".to_string(),
        };
    }
    let mut unknown = required.uncertain;
    for candidate in &candidates {
        match signature_coverage(required, candidate, index) {
            Some(true) => {
                return ApiFinding {
                    offset: required.offset,
                    status: ApiStatus::Covered,
                    signature: display,
                    detail: format!("covered by {}", candidate.display()),
                };
            }
            Some(false) => {}
            None => unknown = true,
        }
    }
    if candidates.is_empty() && index.types.contains(&required.callable) {
        unknown = true;
    }
    ApiFinding {
        offset: required.offset,
        status: if unknown {
            ApiStatus::Unknown
        } else {
            ApiStatus::Missing
        },
        signature: display,
        detail: if unknown {
            "static types or generated methods are insufficient to decide".to_string()
        } else if candidates.is_empty() {
            "no method with this callable is declared".to_string()
        } else {
            "no declared method covers this signature".to_string()
        },
    }
}

fn declaration_finding(
    declaration: &MethodSignature,
    index: &ApiIndex,
    package_name: &str,
) -> ApiFinding {
    let display = declaration.display();
    let owner = signature_owner(declaration, package_name);
    if declaration
        .qualifier
        .as_deref()
        .is_some_and(|qualifier| qualifier.contains('.'))
    {
        return ApiFinding {
            offset: declaration.offset,
            status: ApiStatus::Unknown,
            signature: display,
            detail: "nested module ownership cannot be resolved statically".to_string(),
        };
    }
    if owner == package_name {
        return ApiFinding {
            offset: declaration.offset,
            status: ApiStatus::Planned,
            signature: display,
            detail: "declares a package-owned design signature".to_string(),
        };
    }
    if !index.dependencies.contains(&owner) {
        return ApiFinding {
            offset: declaration.offset,
            status: ApiStatus::Missing,
            signature: display,
            detail: format!("{owner} is not a direct Manifest-bound dependency"),
        };
    }
    if let Some(reason) = index.unresolved_modules.get(&owner) {
        return ApiFinding {
            offset: declaration.offset,
            status: ApiStatus::Unknown,
            signature: display,
            detail: reason.clone(),
        };
    }
    if !index
        .bindings
        .contains(&(owner.clone(), declaration.callable.clone()))
    {
        return ApiFinding {
            offset: declaration.offset,
            status: ApiStatus::Missing,
            signature: display,
            detail: format!(
                "{} has no callable or type binding named {}",
                owner, declaration.callable
            ),
        };
    }
    ApiFinding {
        offset: declaration.offset,
        status: ApiStatus::Planned,
        signature: display,
        detail: format!("declares a planned method for the {owner} binding"),
    }
}

fn index_design_hierarchy(index: &mut ApiIndex, root: &SyntaxNode, package_name: &str) {
    for quote in root
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::QUOTE_EXPR)
    {
        let Some(block) = quote
            .children()
            .find(|child| child.kind() == SyntaxKind::BLOCK)
        else {
            continue;
        };
        for statement in block.children().filter(|statement| {
            statement.kind() == SyntaxKind::BINARY_EXPR && has_token(statement, SyntaxKind::SUBTYPE)
        }) {
            let parts = children(&statement);
            if parts.len() != 2 {
                continue;
            }
            let Some(child) = declaration_name(&parts[0]) else {
                continue;
            };
            let parent = type_spec(&parts[1]).text;
            index.parents.insert(child.clone(), parent);
            index.types.insert(child.clone());
            index.bindings.insert((package_name.to_string(), child));
        }
    }
}

fn collect_julia_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            collect_julia_files(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "jl") {
            files.push(path);
        }
    }
    Ok(())
}

fn quoted_value(value: &str) -> Option<String> {
    let value = value.trim();
    value
        .strip_prefix('"')?
        .split_once('"')
        .map(|(value, _)| value.to_string())
}

fn project_name(package_root: &Path) -> Option<String> {
    let project = fs::read_to_string(package_root.join("Project.toml")).ok()?;
    let mut in_section = false;
    for line in project.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = true;
        }
        if !in_section
            && let Some((key, value)) = line.split_once('=')
            && key.trim() == "name"
        {
            return quoted_value(value);
        }
    }
    None
}

fn project_dependencies(package_root: &Path) -> Result<HashMap<String, String>, String> {
    let path = package_root.join("Project.toml");
    let project =
        fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut dependencies = HashMap::new();
    let mut in_dependencies = false;
    for line in project.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_dependencies = line == "[deps]";
            continue;
        }
        if in_dependencies
            && let Some((name, value)) = line.split_once('=')
            && let Some(uuid) = quoted_value(value)
        {
            dependencies.insert(name.trim().to_string(), uuid);
        }
    }
    Ok(dependencies)
}

#[derive(Clone, Debug, Default)]
struct ManifestDependency {
    name: String,
    uuid: Option<String>,
    version: Option<String>,
    path: Option<String>,
    tree: Option<String>,
}

fn manifest_dependencies(package_root: &Path) -> Result<Vec<ManifestDependency>, String> {
    let path = package_root.join("Manifest.toml");
    let manifest =
        fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(parse_manifest_dependencies(&manifest))
}

fn parse_manifest_dependencies(manifest: &str) -> Vec<ManifestDependency> {
    let mut records = Vec::new();
    let mut current: Option<ManifestDependency> = None;
    for line in manifest.lines() {
        let line = line.trim();
        if let Some(name) = line
            .strip_prefix("[[deps.")
            .and_then(|header| header.strip_suffix("]]"))
        {
            if let Some(record) = current.take() {
                records.push(record);
            }
            current = Some(ManifestDependency {
                name: name.to_string(),
                ..ManifestDependency::default()
            });
            continue;
        }
        if line.starts_with("[[") {
            if let Some(record) = current.take() {
                records.push(record);
            }
            continue;
        }
        let Some(record) = current.as_mut() else {
            continue;
        };
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = quoted_value(value);
        match key.trim() {
            "uuid" => record.uuid = value,
            "version" => record.version = value,
            "path" => record.path = value,
            "git-tree-sha1" => record.tree = value,
            _ => {}
        }
    }
    if let Some(record) = current {
        records.push(record);
    }
    records
}

fn project_identity(package_root: &Path) -> (Option<String>, Option<String>, Option<String>) {
    let Ok(project) = fs::read_to_string(package_root.join("Project.toml")) else {
        return (None, None, None);
    };
    let mut name = None;
    let mut uuid = None;
    let mut version = None;
    for line in project
        .lines()
        .take_while(|line| !line.trim().starts_with('['))
    {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "name" => name = quoted_value(value),
            "uuid" => uuid = quoted_value(value),
            "version" => version = quoted_value(value),
            _ => {}
        }
    }
    (name, uuid, version)
}

fn julia_depots() -> Vec<PathBuf> {
    if let Some(value) = std::env::var_os("JULIA_DEPOT_PATH") {
        let depots: Vec<_> = std::env::split_paths(&value)
            .filter(|path| !path.as_os_str().is_empty())
            .collect();
        if !depots.is_empty() {
            return depots;
        }
    }
    std::env::var_os("HOME")
        .map(|home| vec![PathBuf::from(home).join(".julia")])
        .unwrap_or_default()
}

fn crc32c(bytes: &[u8], initial: u32) -> u32 {
    let mut checksum = !initial;
    for byte in bytes {
        checksum ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = (checksum & 1).wrapping_neg();
            checksum = (checksum >> 1) ^ (0x82f63b78 & mask);
        }
    }
    !checksum
}

fn hex_bytes(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
        .collect()
}

fn version_slug(uuid: &str, tree: &str, length: usize) -> Option<String> {
    let uuid_value = u128::from_str_radix(&uuid.replace('-', ""), 16).ok()?;
    let mut checksum = crc32c(&uuid_value.to_le_bytes(), 0);
    checksum = crc32c(&hex_bytes(tree)?, checksum);
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut slug = String::with_capacity(length);
    for _ in 0..length {
        slug.push(char::from(
            alphabet[(checksum % alphabet.len() as u32) as usize],
        ));
        checksum /= alphabet.len() as u32;
    }
    Some(slug)
}

fn dependency_source(
    package_root: &Path,
    dependency: &ManifestDependency,
) -> Result<PathBuf, String> {
    if let Some(path) = &dependency.path {
        let path = PathBuf::from(path);
        let path = if path.is_absolute() {
            path
        } else {
            package_root.join(path)
        };
        return path
            .join("src")
            .is_dir()
            .then_some(path)
            .ok_or_else(|| "Manifest path dependency source is unavailable".to_string());
    }
    if let (Some(uuid), Some(tree)) = (&dependency.uuid, &dependency.tree) {
        for depot in julia_depots() {
            for length in [5, 4] {
                let Some(slug) = version_slug(uuid, tree, length) else {
                    return Err("Manifest contains an invalid UUID or git-tree-sha1".to_string());
                };
                let path = depot.join("packages").join(&dependency.name).join(slug);
                let (name, source_uuid, version) = project_identity(&path);
                if name.as_deref() == Some(&dependency.name)
                    && source_uuid.as_deref() == Some(uuid)
                    && dependency
                        .version
                        .as_deref()
                        .is_none_or(|expected| version.as_deref() == Some(expected))
                    && path.join("src").is_dir()
                {
                    return Ok(path);
                }
            }
        }
        return Err(format!(
            "Manifest-pinned source at git-tree-sha1 {tree} is unavailable in JULIA_DEPOT_PATH"
        ));
    }
    let mut candidates = Vec::new();
    for depot in julia_depots() {
        let package_directory = depot.join("packages").join(&dependency.name);
        let Ok(entries) = fs::read_dir(package_directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let (name, uuid, version) = project_identity(&path);
            if name.as_deref() == Some(&dependency.name)
                && dependency
                    .uuid
                    .as_deref()
                    .is_none_or(|expected| uuid.as_deref() == Some(expected))
                && dependency
                    .version
                    .as_deref()
                    .is_none_or(|expected| version.as_deref() == Some(expected))
                && path.join("src").is_dir()
            {
                candidates.push(path);
            }
        }
    }
    candidates.sort();
    candidates.dedup();
    match candidates.as_slice() {
        [path] => Ok(path.clone()),
        [] => Err(format!(
            "Manifest-pinned source{}{} is unavailable in JULIA_DEPOT_PATH",
            dependency
                .version
                .as_deref()
                .map_or(String::new(), |version| format!(" at version {version}")),
            dependency
                .tree
                .as_deref()
                .map_or(String::new(), |tree| format!(" ({tree})"))
        )),
        _ => Err("multiple installed sources match the Manifest record".to_string()),
    }
}

fn read_source_files(package_root: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let source_root = package_root.join("src");
    if !source_root.is_dir() {
        return Err(format!("{} is not a directory", source_root.display()));
    }
    let mut paths = Vec::new();
    collect_julia_files(&source_root, &mut paths)?;
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path)
                .map(|source| (path.clone(), source))
                .map_err(|error| format!("{}: {error}", path.display()))
        })
        .collect()
}

pub fn check_semantics(
    package_root: &Path,
    design_source: &str,
) -> Result<Vec<ApiFinding>, String> {
    let package_name = project_name(package_root)
        .ok_or_else(|| "Project.toml must declare the package name".to_string())?;
    let direct_dependencies = project_dependencies(package_root)?;
    let manifest = if direct_dependencies.is_empty() {
        Vec::new()
    } else {
        manifest_dependencies(package_root)?
    };
    let design = parse(design_source);
    if let Some(diagnostic) = design.diagnostics.first() {
        return Err(format!("Julia parse error: {}", diagnostic.message));
    }
    let signatures = design_signatures(&design.cst);
    let referenced_dependencies: HashSet<_> = signatures
        .iter()
        .filter_map(|signature| {
            signature
                .qualifier
                .as_deref()
                .and_then(|qualifier| qualifier.split('.').next())
                .filter(|owner| *owner != package_name)
                .map(str::to_string)
        })
        .collect();
    let mut index = ApiIndex::default();
    index
        .dependencies
        .extend(direct_dependencies.keys().cloned());
    for (name, uuid) in direct_dependencies
        .iter()
        .filter(|(name, _)| referenced_dependencies.contains(*name))
    {
        index.dependencies.insert(name.clone());
        let records: Vec<_> = manifest
            .iter()
            .filter(|record| record.name == *name && record.uuid.as_deref() == Some(uuid))
            .collect();
        let record = match records.as_slice() {
            [record] => *record,
            [] => {
                index.unresolved_modules.insert(
                    name.clone(),
                    "direct dependency has no matching Manifest record".to_string(),
                );
                continue;
            }
            _ => {
                index.unresolved_modules.insert(
                    name.clone(),
                    "direct dependency has multiple matching Manifest records".to_string(),
                );
                continue;
            }
        };
        match dependency_source(package_root, record) {
            Ok(root) => match read_source_files(&root) {
                Ok(files) => {
                    if let Err(error) = index_source_files(&mut index, &files, name) {
                        index.unresolved_modules.insert(name.clone(), error);
                    }
                }
                Err(error) => {
                    index.unresolved_modules.insert(name.clone(), error);
                }
            },
            Err(error) => {
                index.unresolved_modules.insert(name.clone(), error);
            }
        }
    }
    index_design_hierarchy(&mut index, &design.cst, &package_name);

    for declaration in signatures
        .iter()
        .filter(|signature| signature.origin == SignatureOrigin::Declaration)
    {
        let finding = declaration_finding(declaration, &index, &package_name);
        if finding.status.is_compatible() {
            index_method(&mut index, declaration.clone(), &package_name);
        }
    }

    Ok(signatures
        .iter()
        .map(|signature| match signature.origin {
            SignatureOrigin::Declaration => declaration_finding(signature, &index, &package_name),
            SignatureOrigin::Requirement => {
                compare_signature(signature, &index, Some(&package_name))
            }
            SignatureOrigin::Implementation => unreachable!(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn semantic_fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/semantic-package")
    }

    fn compare(design: &str, source: &str) -> Vec<ApiStatus> {
        let source_files = vec![(PathBuf::from("src/PackageName.jl"), source.to_string())];
        let index = source_index(&source_files, "PackageName").unwrap();
        let parsed = parse(design);
        design_signatures(&parsed.cst)
            .iter()
            .map(|signature| compare_signature(signature, &index, Some("PackageName")).status)
            .collect()
    }

    #[test]
    fn reports_exact_covered_missing_and_unknown_signatures() {
        let source = r#"
module PackageName
abstract type AbstractInput end
struct Input <: AbstractInput end

function exact(input::Input, options; mode::Symbol = :fast)::Result
    Result(input, options, mode)
end

function broad(input::AbstractInput, options)::Result
    Result(input, options)
end

function inferred(input::Input)
    Result(input)
end

@generated function generated(input::Input)::Result
    :(Result(input))
end
end
"#;
        let design = r#"
quote
    exact(input::Input, options; mode::Symbol = :fast)::Result
    broad(input::Input, options)::Result
    absent(input::Input)::Result
    inferred(input::Input)::Result
    generated(input::Input)::Result
end
"#;
        assert_eq!(
            compare(design, source),
            vec![
                ApiStatus::Exact,
                ApiStatus::Covered,
                ApiStatus::Missing,
                ApiStatus::Unknown,
                ApiStatus::Unknown,
            ]
        );
    }

    #[test]
    fn checks_long_form_and_qualified_signatures() {
        let source = r#"
module PackageName
function transform!(destination, item::Item)
end
end
"#;
        let design = r#"
quote
    function PackageName.transform!(destination, item::Item)
    end
    function PackageName.transform!(destination, item::OtherItem)
    end
    function PackageName.fabricated(item::Item)
    end
    function OtherPackage.transform!(destination, item::Item)
    end
end
"#;
        assert_eq!(
            compare(design, source),
            vec![
                ApiStatus::Exact,
                ApiStatus::Missing,
                ApiStatus::Missing,
                ApiStatus::Missing,
            ]
        );
    }

    #[test]
    fn parses_manifest_pins() {
        let manifest = r#"
julia_version = "1.12.7"
manifest_format = "2.0"

[[deps.Dependency]]
git-tree-sha1 = "0123456789abcdef"
uuid = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
version = "2.3.4"

[[deps.PathDependency]]
path = "../PathDependency"
uuid = "11111111-2222-3333-4444-555555555555"
version = "1.0.0"
"#;
        let records = parse_manifest_dependencies(manifest);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Dependency");
        assert_eq!(records[0].version.as_deref(), Some("2.3.4"));
        assert_eq!(records[0].tree.as_deref(), Some("0123456789abcdef"));
        assert_eq!(records[1].path.as_deref(), Some("../PathDependency"));
    }

    #[test]
    fn reproduces_julia_manifest_source_slug() {
        assert_eq!(
            version_slug(
                "56664e29-41e4-4ea5-ab0e-825499acc647",
                "f3bc74cec11b204f582df4191f2ddb2ca4d39368",
                5,
            )
            .as_deref(),
            Some("Cqcf1")
        );
    }

    #[test]
    fn scopes_qualified_signatures_to_direct_dependencies() {
        let root_files = vec![(
            PathBuf::from("src/PackageName.jl"),
            "module PackageName\nfunction transform(x::RootInput) end\nend\n".to_string(),
        )];
        let dependency_files = vec![(
            PathBuf::from("Dependency/src/Dependency.jl"),
            "module Dependency\nfunction transform(x::DependencyInput) end\nend\n".to_string(),
        )];
        let mut index = source_index(&root_files, "PackageName").unwrap();
        index.dependencies.insert("Dependency".to_string());
        index_source_files(&mut index, &dependency_files, "Dependency").unwrap();
        let design = parse(
            "quote\nDependency.transform(x::DependencyInput)\nOther.transform(x::Input)\nend\n",
        );
        let statuses: Vec<_> = design_signatures(&design.cst)
            .iter()
            .map(|signature| compare_signature(signature, &index, Some("PackageName")).status)
            .collect();
        assert_eq!(statuses, vec![ApiStatus::Exact, ApiStatus::Missing]);
    }

    #[test]
    fn checks_a_design_without_package_source() {
        let root = semantic_fixture();
        assert!(!root.join("src").exists());
        let design = fs::read_to_string(root.join("docs/design/IdiomaticJulia.jl")).unwrap();
        let statuses: Vec<_> = check_semantics(&root, &design)
            .unwrap()
            .into_iter()
            .map(|finding| finding.status)
            .collect();
        assert_eq!(statuses, vec![ApiStatus::Planned, ApiStatus::Exact]);
    }

    #[test]
    fn checks_requirements_against_the_planned_dispatch_surface() {
        let design = r#"
quote
    SpecificInput <: GeneralInput
    transform(input::GeneralInput)::Result
    result::Result = transform(input::SpecificInput)
    transform(input)
end
"#;
        let statuses: Vec<_> = check_semantics(&semantic_fixture(), design)
            .unwrap()
            .into_iter()
            .map(|finding| finding.status)
            .collect();
        assert_eq!(
            statuses,
            vec![ApiStatus::Planned, ApiStatus::Covered, ApiStatus::Covered]
        );
    }

    #[test]
    fn checks_requirements_inside_control_flow() {
        let design = r#"
quote
    available(input::Input)::Bool
    records(input::Input)::Items
    transform!(item::Item, options::Options)

    if available(input)
        for item in records(input)
            transform!(item, options)
        end
    end
end
"#;
        let statuses: Vec<_> = check_semantics(&semantic_fixture(), design)
            .unwrap()
            .into_iter()
            .map(|finding| finding.status)
            .collect();
        assert_eq!(
            statuses,
            vec![
                ApiStatus::Planned,
                ApiStatus::Planned,
                ApiStatus::Planned,
                ApiStatus::Covered,
                ApiStatus::Covered,
                ApiStatus::Covered,
            ]
        );
    }

    #[test]
    fn validates_dependency_extensions_and_requirements_differently() {
        let design = r#"
quote
    Dependency.external(input::OtherInput)::ExternalResult
    Dependency.fabricated(input::ExternalInput)::ExternalResult
    result::ExternalResult = Dependency.external(input::OtherInput)
end
"#;
        let findings = check_semantics(&semantic_fixture(), design).unwrap();
        let statuses: Vec<_> = findings.iter().map(|finding| finding.status).collect();
        assert_eq!(
            statuses,
            vec![ApiStatus::Planned, ApiStatus::Missing, ApiStatus::Exact]
        );
    }
}
