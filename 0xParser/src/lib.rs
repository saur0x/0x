use std::str::FromStr;

use crate::string_utils::StringUtils;
use regex::Regex;

pub mod string_utils;

pub type Parser<T> = Box<dyn Fn(Context) -> Result<Success<T>, Failure>>;

#[derive(Debug, Clone)]
pub struct Context {
    pub txt: String,
    pub pos: usize,
}

#[derive(Debug, Clone)]
pub struct Success<T: std::fmt::Debug + Clone> {
    pub val: T,
    pub ctx: Context,
}

#[derive(Debug, Clone)]
pub struct Failure {
    pub exp: String,
    pub ctx: Context,
}

pub fn success<T: std::fmt::Debug + Clone>(ctx: Context, val: T) -> Success<T> {
    Success { val, ctx }
}

pub fn failure(ctx: Context, exp: String) -> Failure {
    Failure { exp, ctx }
}

pub fn string(target: String) -> Parser<String> {
    Box::new(move |mut ctx: Context| {
        if ctx.txt.slice(ctx.pos..).starts_with(&target.clone()) {
            ctx.pos += target.len();
            return Ok(success(ctx, target.clone()));
        }

        return Err(failure(ctx, target.clone()));
    })
}

pub fn regex(target: String, expected: String) -> Parser<String> {
    Box::new(move |mut ctx: Context| {
        let regex = match Regex::new(&target.clone()) {
            Ok(regex) => regex,
            Err(_) => panic!("Invalid regex: {}", target),
        };

        let sliced_ctx = ctx.txt.slice(ctx.pos..);
        let mat = regex.find(&sliced_ctx);
        if mat.is_some() {
            if mat.unwrap().start() == 0 {
                ctx.pos += mat.unwrap().end();
                return Ok(success(ctx, mat.unwrap().as_str().to_string()));
            }
        }

        return Err(failure(ctx, expected.clone()));
    })
}

pub fn optional<T: std::fmt::Debug + Clone + 'static>(parser: Parser<T>) -> Parser<Option<T>> {
    Box::new(move |ctx: Context| {
        let res = parser(ctx.clone());

        if res.is_err() {
            return Ok(success(res.unwrap_err().ctx, None));
        }

        return Ok(success(res.clone().unwrap().ctx, Some(res.unwrap().val)));
    })
}

pub fn sequence<T: std::fmt::Debug + Clone + 'static, U: std::fmt::Debug + Clone + 'static>(
    a: Parser<T>,
    b: Parser<U>,
) -> Parser<(T, U)> {
    Box::new(move |mut ctx: Context| {
        let res_a = a(ctx.clone());
        if res_a.is_err() {
            return Err(res_a.unwrap_err());
        }
        ctx = res_a.clone().unwrap().ctx;

        let res_b = b(ctx.clone());
        if res_b.is_err() {
            return Err(res_b.unwrap_err());
        }
        ctx = res_b.clone().unwrap().ctx;

        return Ok(success(ctx, (res_a.unwrap().val, res_b.unwrap().val)));
    })
}

pub fn any<T: std::fmt::Debug + Clone + 'static>(parsers: Vec<Parser<T>>) -> Parser<T> {
    Box::new(move |ctx: Context| {
        for parser in parsers.iter() {
            let res = parser(ctx.clone());
            if res.is_ok() {
                return res;
            }
        }

        return Err(failure(ctx, String::from("any()")));
    })
}

pub fn map<T: std::fmt::Debug + Clone + 'static, U: std::fmt::Debug + Clone + 'static>(
    parser: Parser<T>,
    mapper: fn(T) -> Result<U, String>,
) -> Parser<U> {
    Box::new(move |ctx: Context| {
        let res = parser(ctx.clone());
        if res.is_err() {
            return Err(res.unwrap_err());
        }

        let ctx = res.clone().unwrap().ctx.clone();
        let new_res = mapper(res.unwrap().val);
        if new_res.is_ok() {
            return Ok(success(ctx, new_res.unwrap()));
        }

        return Err(failure(ctx, new_res.unwrap_err()));
    })
}

pub fn many<T: std::fmt::Debug + Clone + 'static>(parser: Parser<T>) -> Parser<Vec<T>> {
    Box::new(move |mut ctx: Context| {
        let mut ret: Vec<T> = Vec::new();

        loop {
            let res = parser(ctx.clone());

            if res.is_err() {
                if ret.len() == 0 {
                    return Err(failure(res.clone().unwrap_err().ctx, res.unwrap_err().exp));
                }

                return Ok(success(ctx, ret));
            }

            ctx = res.clone().unwrap().ctx;
            ret.push(res.unwrap().val);
        }
    })
}

pub fn spaces() -> Parser<String> {
    return map(many(string(" ".to_string())), |s: Vec<String>| {
        Ok(s.join(""))
    });
}

pub fn integer() -> Parser<String> {
    return regex(r"\d+".to_string(), "integer".to_string());
}

pub fn parsed_integer<T: std::fmt::Debug + Clone + 'static + FromStr>() -> Parser<T> {
    return map(
        regex(r"\d+".to_string(), "integer".to_string()),
        |s: String| match s.parse::<T>() {
            Ok(val) => Ok(val),
            Err(_) => Err("parsable integer".to_string()),
        },
    );
}

pub fn float() -> Parser<String> {
    return regex(r"\d+\.\d*".to_string(), "float".to_string());
}

pub fn parsed_float<T: std::fmt::Debug + Clone + 'static + FromStr>() -> Parser<T> {
    return map(
        regex(r"\d+\.\d*".to_string(), "float".to_string()),
        |s: String| match s.parse::<T>() {
            Ok(val) => Ok(val),
            Err(_) => Err("parsable float".to_string()),
        },
    );
}

pub fn expect<T: std::fmt::Debug + Clone + 'static>(
    parser: Parser<T>,
    expected: String,
) -> Parser<T> {
    Box::new(move |ctx: Context| {
        let res = parser(ctx.clone());
        if res.is_err() {
            return Err(failure(res.unwrap_err().ctx, expected.clone()));
        }

        return res;
    })
}

pub fn parse<T: std::fmt::Debug + Clone + 'static>(
    txt: String,
    parser: Parser<T>,
) -> Result<T, String> {
    let res = parser(Context { txt, pos: 0 });
    if res.is_err() {
        return Err(format!(
            "Parser error, expected '{}' at position '{}'",
            res.clone().unwrap_err().exp,
            res.unwrap_err().ctx.pos
        ));
    }

    return Ok(res.unwrap().val);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_test() {
        let res = parse("Hello World".to_string(), string("Hello World".to_string()));
        assert_eq!(res.unwrap(), "Hello World".to_string());

        let res = parse("Hello World".to_string(), string("Hallo World".to_string()));
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'Hallo World' at position '0'"
        );

        let res = parse(
            "My Hello World".to_string(),
            string("Hello World".to_string()),
        );
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'Hello World' at position '0'"
        );
    }

    #[test]
    fn regex_test() {
        let res = parse(
            "DE0012 2322 2323".to_string(),
            regex(r"DE\d{4}\s\d{4}\s\d{4}".to_string(), "IBAN".to_string()),
        );
        assert_eq!(res.unwrap(), "DE0012 2322 2323".to_string());

        let res = parse(
            "DE012 2322 2323".to_string(),
            regex(r"DE\d{4}\s\d{4}\s\d{4}".to_string(), "IBAN".to_string()),
        );
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'IBAN' at position '0'"
        );

        let res = parse(
            "Bank account: DE012 2322 2323".to_string(),
            regex(r"DE\d{4}\s\d{4}\s\d{4}".to_string(), "IBAN".to_string()),
        );
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'IBAN' at position '0'"
        );
    }

    #[test]
    fn optional_test() {
        let res = parse(
            "Hello World".to_string(),
            optional(string("Hello World".to_string())),
        );
        assert_eq!(res.unwrap(), Some("Hello World".to_string()));

        let res = parse(
            "Hello World".to_string(),
            optional(string("Hallo World".to_string())),
        );
        assert_eq!(res.unwrap(), None);
    }

    #[test]
    fn sequence_test() {
        let res = parse(
            "Hello World".to_string(),
            sequence(string("Hello".to_string()), string(" World".to_string())),
        );
        assert_eq!(res.unwrap(), ("Hello".to_string(), " World".to_string()));

        let res = parse(
            "Hello World".to_string(),
            sequence(string("Hallo".to_string()), string(" World".to_string())),
        );
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'Hallo' at position '0'"
        );

        let res = parse(
            "Hello World".to_string(),
            sequence(string("Hello".to_string()), string("World".to_string())),
        );
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'World' at position '5'"
        );

        let res = parse(
            "Hello World".to_string(),
            sequence(
                sequence(string("Hello".to_string()), string(" ".to_string())),
                string("World".to_string()),
            ),
        );
        assert_eq!(
            res.unwrap(),
            (("Hello".to_string(), " ".to_string()), "World".to_string())
        );
    }

    #[test]
    fn any_test() {
        let res = parse(
            "Hello World".to_string(),
            sequence(
                any(vec![
                    string("Hallo".to_string()),
                    string("Hello".to_string()),
                ]),
                string(" World".to_string()),
            ),
        );

        assert_eq!(res.unwrap(), ("Hello".to_string(), " World".to_string()));

        let res = parse(
            "Hello World".to_string(),
            sequence(
                any(vec![
                    string("Hallo".to_string()),
                    string("Hola".to_string()),
                ]),
                string(" World".to_string()),
            ),
        );

        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'any()' at position '0'"
        );
    }

    #[test]
    fn map_test() {
        let res = parse(
            "Hello World".to_string(),
            map(
                sequence(
                    sequence(string("Hello".to_string()), string(" ".to_string())),
                    string("World".to_string()),
                ),
                |res| Ok((res.0 .0, res.0 .1, res.1)),
            ),
        );
        assert_eq!(
            res.unwrap(),
            ("Hello".to_string(), " ".to_string(), "World".to_string())
        );

        let res = parse::<Option<String>>(
            "Hello World".to_string(),
            map(
                sequence(
                    sequence(string("Hello".to_string()), string(" ".to_string())),
                    string("World".to_string()),
                ),
                |_| Err("mapping()".to_string()),
            ),
        );
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'mapping()' at position '11'"
        );
    }

    #[test]
    fn many_test() {
        let res = parse(
            "Hello World".to_string(),
            many(regex(r".{1}".to_string(), "anything".to_string())),
        );
        assert_eq!(res.unwrap().join(""), "Hello World".to_string());

        let res = parse(
            "Hello World".to_string(),
            many(regex(r"\d{1}".to_string(), "number".to_string())),
        );
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'number' at position '0'"
        );
    }

    #[test]
    fn spaces_test() {
        let res = parse(
            "Hello World".to_string(),
            sequence(
                sequence(string("Hello".to_string()), spaces()),
                string("World".to_string()),
            ),
        );
        assert_eq!(
            res.unwrap(),
            (("Hello".to_string(), " ".to_string()), "World".to_string())
        );

        let res = parse(
            "HelloWorld".to_string(),
            sequence(
                sequence(string("Hello".to_string()), spaces()),
                string("World".to_string()),
            ),
        );
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected ' ' at position '5'"
        );

        let res = parse(
            "Hello    World".to_string(),
            sequence(
                sequence(string("Hello".to_string()), spaces()),
                string("World".to_string()),
            ),
        );
        assert_eq!(
            res.unwrap(),
            (
                ("Hello".to_string(), "    ".to_string()),
                "World".to_string()
            )
        );
    }

    #[test]
    fn integer_test() {
        let res = parse("123456789".to_string(), integer());
        assert_eq!(res.unwrap(), "123456789");

        let res = parse("a123456789".to_string(), integer());
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'integer' at position '0'"
        );
    }

    #[test]
    fn parsed_integer_test() {
        let res = parse("123456789".to_string(), parsed_integer::<i32>());
        assert_eq!(res.unwrap(), 123456789i32);

        let res = parse("123456789".to_string(), parsed_integer::<u64>());
        assert_eq!(res.unwrap(), 123456789u64);

        let res = parse("123456789".to_string(), parsed_integer::<u8>());
        // bad error for impossible to parse value
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'parsable integer' at position '9'"
        );

        let res = parse("a123456789".to_string(), parsed_integer::<u32>());
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'integer' at position '0'"
        );
    }

    #[test]
    fn float_test() {
        let res = parse("12345.6789".to_string(), float());
        assert_eq!(res.unwrap(), "12345.6789");

        let res = parse("a1234.56789".to_string(), float());
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'float' at position '0'"
        );
    }

    #[test]
    fn parsed_float_test() {
        let res = parse("12345.6789".to_string(), parsed_float::<f32>());
        assert_eq!(res.unwrap(), 12345.6789f32);

        let res = parse("12345678.9".to_string(), parsed_float::<f64>());
        assert_eq!(res.unwrap(), 12345678.9f64);

        let res = parse("a12345.6789".to_string(), parsed_float::<f32>());
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected 'float' at position '0'"
        );
    }

    #[test]
    fn expect_test() {
        let res = parse(
            "Hello World".to_string(),
            expect(string("Hello".to_string()), "\"Hello\"".to_string()),
        );
        assert_eq!(res.unwrap(), "Hello".to_string());

        let res = parse(
            "Hello World".to_string(),
            expect(string("Hallo".to_string()), "\"Hallo\"".to_string()),
        );
        assert_eq!(
            res.unwrap_err(),
            "Parser error, expected '\"Hallo\"' at position '0'".to_string()
        );
    }
}
