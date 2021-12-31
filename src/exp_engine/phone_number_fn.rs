use std::str::FromStr;

use jmespatch::{Context, JmespathError, Rcvar, Variable};
use jmespatch::functions::{ArgumentType, Function, Signature};
use lazy_static::lazy_static;
use phonenumber::country::Id;
use serde_json::{json, Value};

lazy_static! {
    static ref NULL_RC: Rcvar = Rcvar::new(Variable::Null);
}


pub struct PhoneNumberFn {
    signature: Signature,
}

impl Default for PhoneNumberFn {
    fn default() -> Self {
        PhoneNumberFn::new()
    }
}

impl PhoneNumberFn {
    pub fn new() -> PhoneNumberFn {
        PhoneNumberFn {
            signature: Signature::new(
                vec![
                    ArgumentType::Any
                ],
                Some(
                    ArgumentType::Union(
                        vec![
                            ArgumentType::TypedArray(Box::new(ArgumentType::String))
                        ]
                    )
                ),
            )
        }
    }

    fn parse_number(country: Option<Id>, number: String) -> Result<Value, phonenumber::ParseError> {
        match phonenumber::parse(country, number) {
            Ok(n) => {
                match n.is_valid() {
                    true => Ok(json!({
                        "from": country,
                        "country": n.country().id(),
                        "country_code": n.country().code(),
                        "national": n.national().value(),
                        "national_zeros": n.national().zeros(),
                        "valid": n.is_valid(),
                        "carrier": n.carrier()
                    })),
                    false => Err(phonenumber::ParseError::NoNumber)
                }
            }
            Err(e) => Err(e)
        }
    }

    fn id_from_str(s: &String) -> Option<Id> {
        Id::from_str(s.to_uppercase().as_str())
            .map_or_else(|_| None, |id| Some(id))
    }

    fn load_countries(var: &Variable) -> Vec<Option<Id>> {
        match var {
            Variable::Array(a) => a.iter().map(
                |s| Self::id_from_str(s.as_string().unwrap_or(&"".to_string()))
            ).collect(),
            _ => vec![]
        }
    }

    fn match_number_on(num: &String, countries: Vec<Option<Id>>, single: bool) -> Option<Rcvar> {
        let mut res: Vec<Rcvar> = vec![];
        for country in countries {
            let parsed = match Self::parse_number(country.clone(), num.to_owned()) {
                Ok(info) => match Variable::try_from(info) {
                    Ok(r) => r,
                    Err(_) => Variable::Null
                },
                Err(_) => Variable::Null
            };

            if !parsed.is_null() {
                if single {
                    return Some(Rcvar::new(parsed));
                }

                res.push(Rcvar::new(parsed))
            }
        }

        if res.len() == 0 {
            Some(Rcvar::new(Variable::Null))
        } else {
            Some(Rcvar::new(Variable::Array(res)))
        }
    }
}

impl Function for PhoneNumberFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let countries: Vec<Option<Id>> = if args.len() > 1 {
            Self::load_countries(args[1].as_ref())
        } else {
            vec![None]
        };

        return Ok(match args[0].as_string() {
            None => NULL_RC.clone(),
            Some(num) => Self::match_number_on(
                num,
                countries,
                args.len() <= 1,
            ).unwrap_or(NULL_RC.clone())
        });
    }
}


#[cfg(test)]
mod phone_number_tests {
    use serde_json::json;

    use crate::exp_engine::phone_number_fn::PhoneNumberFn;
    use crate::runtime::compile_expr;

    fn test_number(payload: &str, country: &str) {
        let exp = compile_expr(format!("phone_number(@, {})", country)).unwrap();
        let r = exp.search(json!(payload)).unwrap();
        assert!(r.is_truthy());
        println!("phone_number({}, {}) = {}", payload, country, r);
    }

    fn test_wrong_number(payload: &str, country: &str) {
        let exp = compile_expr(format!("phone_number(@, {})", country)).unwrap();
        let r = exp.search(json!(payload)).unwrap();
        assert!(!r.is_truthy());
        println!("phone_number({}, {}) = {}", payload, country, r);
    }

    #[test]
    fn test_is_phone_number_is_parsed() {
        let r = PhoneNumberFn::parse_number(
            None,
            "+1-541-754-3010".to_string(),
        );
        assert!(!r.is_err());
    }

    #[test]
    fn test_is_valid_phone_number_returns_non_null() {
        let nz = "['nz']";
        test_number("033316005", &nz);
        test_number("03-331 6005", &nz);
        test_number("0064 3 331 6005", &nz);

        let countries = "['us', 'fr', 'de']";
        test_number("(541) 754-3010", &countries);
        test_number("1-541-754-3010", &countries);
        test_number("01 45 45 32 45", &countries);
        test_number("(089) / 636-48018", &countries);
    }

    #[test]
    fn test_not_valid_phone_number_returns_null() {
        let nz = "['us']";
        test_wrong_number("033316005", &nz);
        test_wrong_number("03-331 6005", &nz);
        test_wrong_number("0064 3 331 6005", &nz);

        let countries = "['nz', 'ch']";
        test_wrong_number("(541) 754-3010", &countries);
        test_wrong_number("1-541-754-3010", &countries);
        test_wrong_number("01 45 45 32 45", &countries);
        test_wrong_number("(089) / 636-48018", &countries);
    }

    #[test]
    fn test_is_valid_intl_phone_number_returns_non_null() {
        let exp = compile_expr(format!("phone_number(@)")).unwrap();
        let r = exp.search(json!("+33-655-562-025")).unwrap();
        assert!(r.is_truthy());
        println!("Intl OK = {}", r)
    }

    #[test]
    fn test_not_valid_intl_phone_number_returns_null() {
        let exp = compile_expr(format!("phone_number(@)")).unwrap();
        let r = exp.search(json!("33655562025")).unwrap();
        assert!(!r.is_truthy());
    }
}