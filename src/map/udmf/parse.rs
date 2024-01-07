use winnow::{
    ascii::{dec_int, dec_uint, escaped_transform, float, hex_uint, Caseless},
    combinator::{alt, cut_err, delimited, eof, preceded, repeat, repeat_till0, rest, terminated},
    token::{one_of, take_till, take_while},
    Located, PResult, Parser,
};

use crate::map::udmf::{ast, Identifier, Value};

pub fn parse_translation_unit(input: &mut Located<&str>) -> PResult<ast::TranslationUnit> {
    let (expressions, _) = repeat_till0(
        alt((
            parse_block
                .with_span()
                .map(ast::Spanned::wrap)
                .map(ast::GlobalExpr::Block),
            parse_assignment_expr
                .with_span()
                .map(ast::Spanned::wrap)
                .map(ast::GlobalExpr::AssignmentExpr),
        )),
        (parse_whitespace_and_comments, eof),
    )
    .parse_next(input)?;

    Ok(ast::TranslationUnit { expressions })
}

fn parse_block(input: &mut Located<&str>) -> PResult<ast::Block> {
    let _wc = parse_whitespace_and_comments.parse_next(input)?;
    let identifier = parse_identifier
        .with_span()
        .map(ast::Spanned::wrap)
        .parse_next(input)?;

    let _wc = parse_whitespace_and_comments.parse_next(input)?;
    let _brace = '{'.parse_next(input)?;

    let assignments = repeat(
        0..,
        parse_assignment_expr.with_span().map(ast::Spanned::wrap),
    )
    .parse_next(input)?;

    let _wc = parse_whitespace_and_comments.parse_next(input)?;
    let _brace = '}'.parse_next(input)?;

    Ok(ast::Block {
        identifier,
        assignments,
    })
}

fn parse_value(input: &mut Located<&str>) -> PResult<Value> {
    alt((
        parse_integer.map(Value::Int),
        parse_float.map(Value::Float),
        parse_quoted_string.map(Value::Str),
        parse_bool.map(Value::Bool),
    ))
    .parse_next(input)
}

fn parse_assignment_expr(input: &mut Located<&str>) -> PResult<ast::AssignmentExpr> {
    let _wc = parse_whitespace_and_comments.parse_next(input)?;
    let identifier = parse_identifier
        .with_span()
        .map(ast::Spanned::wrap)
        .parse_next(input)?;

    let _wc = parse_whitespace_and_comments.parse_next(input)?;
    let _equals = '='.parse_next(input)?;

    let _wc = parse_whitespace_and_comments.parse_next(input)?;
    let value = parse_value
        .with_span()
        .map(ast::Spanned::wrap)
        .parse_next(input)?;

    let _wc = parse_whitespace_and_comments.parse_next(input)?;
    let _semicolon = ';'.parse_next(input)?;

    Ok(ast::AssignmentExpr { identifier, value })
}

fn parse_integer(input: &mut Located<&str>) -> PResult<i32> {
    alt((
        dec_int,
        dec_uint.try_map(|n: u32| i32::try_from(n)),
        preceded("0x", hex_uint.try_map(|n: u32| i32::try_from(n))),
    ))
    .parse_next(input)
}

fn parse_float(input: &mut Located<&str>) -> PResult<f64> {
    float.parse_next(input)
}

fn parse_quoted_string(input: &mut Located<&str>) -> PResult<String> {
    preceded(
        '"',
        cut_err(terminated(
            escaped_transform(
                take_till(0.., &['"', '\\']),
                '\\',
                alt(("\\".value("\\"), "\"".value("\""), "n".value("\n"))),
            ),
            '"',
        )),
    )
    .parse_next(input)
}

fn parse_bool(input: &mut Located<&str>) -> PResult<bool> {
    alt((Caseless("true").value(true), Caseless("false").value(false))).parse_next(input)
}

fn parse_identifier(input: &mut Located<&str>) -> PResult<Identifier> {
    (
        one_of(('a'..='z', 'A'..='Z', '_')),
        take_while(0.., ('a'..='z', 'A'..='Z', '0'..='9', '_')),
    )
        .recognize()
        .map(|s| Identifier(String::from(s)))
        .parse_next(input)
}

fn parse_whitespace_and_comments<'s>(input: &mut Located<&'s str>) -> PResult<&'s str> {
    repeat::<_, _, (), _, _>(
        0..,
        alt((
            parse_line_comment,
            parse_block_comment,
            take_till(1.., |c: char| c.is_whitespace()),
        )),
    )
    .recognize()
    .parse_next(input)
}

fn parse_line_comment<'s>(input: &mut Located<&'s str>) -> PResult<&'s str> {
    preceded("//", alt((take_till(0.., '\n'), rest))).parse_next(input)
}

fn parse_block_comment<'s>(input: &mut Located<&'s str>) -> PResult<&'s str> {
    delimited("/*", take_till(0.., b"*/"), "*/").parse_next(input)
}
