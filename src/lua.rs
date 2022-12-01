use full_moon::{ast, tokenizer};
use std::fs;
use crate::result::Result;
pub fn parse_lua(filename: &str) -> Result<ast::Ast> {
    let code = fs::read_to_string(filename)?;
    Ok(full_moon::parse(&code)?)
}

pub fn find_define_plugins_function<'a>(stmts: &'a [&ast::Stmt]) -> &'a ast::LocalFunction {
    stmts
        .iter()
        .find_map(|s| {
            if let ast::Stmt::LocalFunction(field) = s {
                if field.name().to_string().ends_with("define_plugins") {
                    return Some(field);
                }
            }
            None
        })
        .unwrap()
}
pub fn parse_packer_use_calls(define_plugins: &ast::LocalFunction) -> Vec<&ast::FunctionCall> {
    define_plugins
        .body()
        .block()
        .stmts()
        .filter_map(|inner_stmnt| match inner_stmnt {
            ast::Stmt::FunctionCall(call) => {
                if call.prefix().to_string().trim().ends_with("use") {
                    return Some(call);
                }
                None
            }
            _ => None,
        })
        .filter(|call| does_have_commit_key(call))
        .collect()
}
pub fn update_commit_sha(
    commit_sha: &str,
    existing_token_ref: &tokenizer::TokenReference,
) -> ast::Expression {
    let b = Box::new(ast::Value::String(tokenizer::TokenReference::new(
        existing_token_ref
            .leading_trivia()
            .cloned()
            .collect::<Vec<tokenizer::Token>>(),
        full_moon::tokenizer::Token::new(tokenizer::TokenType::StringLiteral {
            literal: full_moon::ShortString::new(commit_sha),
            multi_line: None,
            quote_type: tokenizer::StringLiteralQuoteType::Double,
        }),
        existing_token_ref
            .trailing_trivia()
            .cloned()
            .collect::<Vec<tokenizer::Token>>(),
    )));
    ast::Expression::Value { value: b }
}

pub fn does_have_commit_key(use_call: &ast::FunctionCall) -> bool {
    match get_table_ctor_for_use_call(use_call) {
        None => false,
        Some(table_ctor) => table_ctor
            .fields()
            .into_iter()
            .filter_map(|field| match field {
                ast::Field::NameKey {
                    key,
                    equal: _,
                    value: _,
                } => Some(key.token().token_type()),
                _ => None,
            })
            .any(|table_ctor_key| match table_ctor_key {
                full_moon::tokenizer::TokenType::Identifier { identifier } => {
                    if identifier.as_str().eq("commit") {
                        return true;
                    }
                    false
                }
                _ => false,
            }),
    }
}

pub fn get_table_ctor_for_use_call(use_call: &ast::FunctionCall) -> Option<&ast::TableConstructor> {
    use_call
        .suffixes()
        .into_iter()
        .filter_map(|suffix| match suffix {
            ast::Suffix::Call(anonymous_call) => Some(anonymous_call),
            _ => None,
        })
        .filter_map(|anonymous_call| match anonymous_call {
            ast::Call::AnonymousCall(ast::FunctionArgs::TableConstructor(table_ctor)) => {
                Some(table_ctor)
            }
            _ => None,
        })
        .next()
}
pub fn get_function_call_by_table_ctor<'a>(
    use_calls: Vec<&'a ast::FunctionCall>,
    table_ctor: &'a ast::TableConstructor,
) -> Option<&'a ast::FunctionCall> {
    use_calls.into_iter().find(|use_call| {
        if let Some(table_ctor_for_call) = get_table_ctor_for_use_call(use_call) {
            return table_ctor_for_call.eq(table_ctor);
        }
        false
    })
}

pub fn get_plugin_name(table_ctor: &ast::TableConstructor) -> Option<&str> {
    table_ctor
        .fields()
        .iter()
        .filter_map(|field| {
            if let ast::Field::NoKey(expression) = field {
                return Some(expression);
            }
            None
        })
        .filter_map(|expression| {
            if let ast::Expression::Value { value } = expression {
                let unboxed = &**value;
                return Some(unboxed);
            }
            None
        })
        .filter_map(|unboxed_value| {
            if let ast::Value::String(token_ref) = unboxed_value {
                return Some(token_ref);
            }
            None
        })
        .filter_map(|token_ref| {
            if let tokenizer::TokenType::StringLiteral {
                literal,
                multi_line: _,
                quote_type: _,
            } = token_ref.token_type()
            {
                return Some(literal.as_str());
            }
            None
        })
        .next()
}
fn traverse_down_to_key<'a, 'b>(
    table_ctor: &'a ast::TableConstructor,
    table_ctor_key: &'b str,
) -> Option<(
    &'a ast::Field,
    &'a tokenizer::TokenReference,
    &'a tokenizer::TokenReference,
    &'a ast::Expression,
    &'a tokenizer::TokenReference,
)> {
    table_ctor
        .fields()
        .iter()
        .filter_map(|field| {
            if let ast::Field::NameKey { key, equal, value } = field {
                return Some((field, key, equal, value));
            }
            None
        })
        .filter_map(|(field, key, equal, expression)| {
            if let full_moon::tokenizer::TokenType::Identifier { identifier } =
                key.token().token_type()
            {
                if identifier.as_str().eq(table_ctor_key) {
                    return Some((field, key, equal, expression));
                }
            }
            None
        })
        .filter_map(|(field, key, equal, expression)| {
            if let ast::Expression::Value { value } = expression {
                let unboxed = &**value;
                return Some((field, key, equal, expression, unboxed));
            }
            None
        })
        .filter_map(|(field, key, equal, expression, unboxed_value)| {
            if let ast::Value::String(token_ref) = unboxed_value {
                return Some((field, key, equal, expression, token_ref));
            }
            None
        })
        .next()
}
pub fn replace_table_constructor(
    table_ctor: &ast::TableConstructor,
    new_commit: &str,
) -> Option<ast::TableConstructor> {
    if let Some((field, key, equal, _expression, token_ref)) =
        traverse_down_to_key(table_ctor, "commit")
    {
        if let full_moon::tokenizer::TokenType::StringLiteral {
            literal: _,
            multi_line: _,
            quote_type: _,
        } = token_ref.token_type()
        {
            let mut punc: ast::punctuated::Punctuated<ast::Field> = table_ctor
                .fields()
                .iter()
                .filter(|f| field.ne(f))
                .map(|f| {
                    ast::punctuated::Pair::Punctuated(
                        f.clone(),
                        tokenizer::TokenReference::symbol(",").unwrap(),
                    )
                })
                .collect();
            let new_sha = update_commit_sha(new_commit, token_ref);
            let new_field = ast::Field::NameKey {
                key: key.clone(),
                equal: equal.clone(),
                value: new_sha,
            };
            punc.push(ast::punctuated::Pair::End(new_field));
            let table_ctor = ast::TableConstructor::new().with_fields(punc);
            return Some(table_ctor);
        }
    }
    None
}

pub fn get_commit(table_ctor: &ast::TableConstructor) -> Option<&str> {
    if let Some((_, _, _, _, token_ref)) = traverse_down_to_key(table_ctor, "commit") {
        if let tokenizer::TokenType::StringLiteral {
            literal,
            multi_line: _,
            quote_type: _,
        } = token_ref.token_type()
        {
            return Some(literal.as_str());
        }
    }
    None
}
pub fn get_branch(table_ctor: &ast::TableConstructor) -> Option<&str> {
    if let Some((_, _, _, _, token_ref)) = traverse_down_to_key(table_ctor, "branch") {
        if let tokenizer::TokenType::StringLiteral {
            literal,
            multi_line: _,
            quote_type: _,
        } = token_ref.token_type()
        {
            return Some(literal.as_str());
        }
    }
    None
}
