/// ==============================================================================
/// MODIFIED VERSION OF:
/// https://github.com/winnow-rs/winnow/blob/main/examples/json/parser_dispatch.rs
/// LICENSE: https://github.com/winnow-rs/winnow/blob/main/COPYRIGHT
/// ==============================================================================
use winnow::{
	ascii::float,
	combinator::{
		alt, cut_err, delimited, dispatch, fail, fold_repeat, peek, preceded, separated0,
		separated_pair, success, terminated,
	},
	error::{AddContext, ParserError},
	token::{any, none_of, take, take_while},
	PResult, Parser,
};

type Obj = Vec<JsonValue>;

#[derive(Clone, Debug)]
pub enum JsonValue {
	Null,
	Boolean(bool),
	Str(String),
	Num(i32),
	Array(Vec<JsonValue>),
	Object(Obj),
}

impl JsonValue {
	pub fn extract_bool(&self) -> bool {
		match self {
			JsonValue::Boolean(b) => *b,
			_ => panic!("not a bool"),
		}
	}
	pub fn extract_str(&self) -> &str {
		match self {
			JsonValue::Str(s) => s,
			_ => panic!("not a string"),
		}
	}
	pub fn extract_num(&self) -> i32 {
		match self {
			JsonValue::Num(i) => *i,
			_ => panic!("not a number"),
		}
	}
	pub fn extract_array(&self) -> &Vec<JsonValue> {
		match self {
			JsonValue::Array(arr) => arr,
			_ => panic!("not an array"),
		}
	}
	pub fn extract_object(&self) -> &Obj {
		match self {
			JsonValue::Object(obj) => obj,
			_ => panic!("not an object"),
		}
	}
	pub fn extract_object_key(&self, idx: usize) -> &JsonValue {
		match self {
			JsonValue::Object(obj) => &obj[idx],
			_ => panic!("not an object"),
		}
	}
}

pub type Stream<'i> = &'i str;

pub fn run<'i, E: ParserError<Stream<'i>> + AddContext<Stream<'i>, &'static str>>(
	input: &mut Stream<'i>,
) -> PResult<JsonValue, E> {
	delimited(ws, json_value, ws).parse_next(input)
}

fn json_value<'i, E: ParserError<Stream<'i>> + AddContext<Stream<'i>, &'static str>>(
	input: &mut Stream<'i>,
) -> PResult<JsonValue, E> {
	dispatch!(peek(any);
		'n' => null.value(JsonValue::Null),
		't' => true_.map(JsonValue::Boolean),
		'f' => false_.map(JsonValue::Boolean),
		'"' => string.map(JsonValue::Str),
		'+' => float.map(JsonValue::Num),
		'-' => float.map(JsonValue::Num),
		'0'..='9' => float.map(JsonValue::Num),
		'[' => array.map(JsonValue::Array),
		'{' => object.map(JsonValue::Object),
		_ => fail,
	)
	.parse_next(input)
}

fn null<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<&'i str, E> {
	"null".parse_next(input)
}

fn true_<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<bool, E> {
	"true".value(true).parse_next(input)
}

fn false_<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<bool, E> {
	"false".value(false).parse_next(input)
}

fn string<'i, E: ParserError<Stream<'i>> + AddContext<Stream<'i>, &'static str>>(
	input: &mut Stream<'i>,
) -> PResult<String, E> {
	preceded(
		'\"',
		cut_err(terminated(
			fold_repeat(0.., character, String::new, |mut string, c| {
				string.push(c);
				string
			}),
			'\"',
		)),
	)
	.context("string")
	.parse_next(input)
}

fn character<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<char, E> {
	let c = none_of('\"').parse_next(input)?;
	if c == '\\' {
		dispatch!(any;
		  '"' => success('"'),
		  '\\' => success('\\'),
		  '/'  => success('/'),
		  'b' => success('\x08'),
		  'f' => success('\x0C'),
		  'n' => success('\n'),
		  'r' => success('\r'),
		  't' => success('\t'),
		  'u' => unicode_escape,
		  _ => fail,
		)
		.parse_next(input)
	} else {
		Ok(c)
	}
}

fn unicode_escape<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<char, E> {
	alt((
		u16_hex
			.verify(|cp| !(0xD800..0xE000).contains(cp))
			.map(|cp| cp as u32),
		separated_pair(u16_hex, "\\u", u16_hex)
			.verify(|(high, low)| (0xD800..0xDC00).contains(high) && (0xDC00..0xE000).contains(low))
			.map(|(high, low)| {
				let high_ten = (high as u32) - 0xD800;
				let low_ten = (low as u32) - 0xDC00;
				(high_ten << 10) + low_ten + 0x10000
			}),
	))
	.verify_map(std::char::from_u32)
	.parse_next(input)
}

fn u16_hex<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<u16, E> {
	take(4usize)
		.verify_map(|s| u16::from_str_radix(s, 16).ok())
		.parse_next(input)
}

fn array<'i, E: ParserError<Stream<'i>> + AddContext<Stream<'i>, &'static str>>(
	input: &mut Stream<'i>,
) -> PResult<Vec<JsonValue>, E> {
	preceded(
		('[', ws),
		cut_err(terminated(separated0(json_value, (ws, ',', ws)), (ws, ']'))),
	)
	.context("array")
	.parse_next(input)
}

fn object<'i, E: ParserError<Stream<'i>> + AddContext<Stream<'i>, &'static str>>(
	input: &mut Stream<'i>,
) -> PResult<Obj, E> {
	preceded(
		('{', ws),
		cut_err(terminated(separated0(key_value, (ws, ',', ws)), (ws, '}'))),
	)
	.context("object")
	.parse_next(input)
}

fn key_value<'i, E: ParserError<Stream<'i>> + AddContext<Stream<'i>, &'static str>>(
	input: &mut Stream<'i>,
) -> PResult<JsonValue, E> {
	separated_pair(string, cut_err((ws, ':', ws)), json_value)
		.map(|(_, v)| v)
		.parse_next(input)
}

fn ws<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<&'i str, E> {
	take_while(0.., WS).parse_next(input)
}

const WS: &[char] = &[' ', '\t', '\r', '\n'];
