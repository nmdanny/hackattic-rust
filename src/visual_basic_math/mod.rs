use super::HackatticChallenge;
use failure::{Error, ResultExt};
use std::io::{Read, Write, BufWriter};
use std::process::{Command, Stdio};
use reqwest;

pub mod ocr;

#[derive(Debug, PartialEq)]
struct Expression {
    operation: Operation,
    number: i32
}

impl Expression {
    fn from_line(line: &str) -> Result<Self, ParseError> {
        let (op, num) = line.split_at(1);
        let operation = Operation::from_str(op).ok_or(ParseError::UnknownOperation { operation: op.to_owned() })?;
        let number = num.parse::<i32>().map_err(ParseError::NotANumber)?;
        Ok(Expression {
            operation, number
        })
    }
    fn from_lines(input: &str) -> Result<Vec<Self>, ParseError> {
        input.trim().lines().map(Expression::from_line).collect()

    }
    fn fold_expressions(exprs: &[Expression]) -> i32 {
        exprs.iter().fold(0, |acc, expr| expr.operation.apply(acc, expr.number))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Operation {
    Plus, Minus, Mult, Div
}

impl Operation {
    fn from_str(c: &str) -> Option<Self> {
        match c {
            "+" => Some(Operation::Plus),
            "-" => Some(Operation::Minus),
            "x" | "×" | "*" => Some(Operation::Mult),
            "/" | "÷"  => Some(Operation::Div),
            _ => None
        }
    }

    fn apply(&self, op0: i32, op1: i32) -> i32 {
        match self {
            &Operation::Plus => op0 + op1,
            &Operation::Minus => op0 - op1,
            &Operation::Mult => op0 * op1,
            &Operation::Div => op0 / op1
        }
    }
}

#[derive(Debug, PartialEq, Fail)]
enum ParseError {
    #[fail(display = "Unknown operation: {}", operation)]
    UnknownOperation { operation: String },
    #[fail(display = "Failed to parse int: {}", _0)]
    NotANumber(::std::num::ParseIntError)
}

#[derive(Deserialize, Debug, Clone)]
struct Problem {
    image_url: String
}

#[derive(Debug, Clone, Serialize)]
struct Solution {
    result: i32
}

pub fn main() {
    main_err().unwrap();
}

pub fn main_err() -> Result<(), Error> {
    VisualBasicMath::process_challenge()?;
    Ok(())
}


struct VisualBasicMath;
impl HackatticChallenge for VisualBasicMath {
    type Problem = Problem;
    type Solution = Solution;
    fn challenge_name() -> &'static str {
        "visual_basic_math"
    }
    fn make_solution(req: &Problem) -> Result<Solution, Error> {
        let mut image_buf = Vec::new();
        reqwest::get(&req.image_url)?.read_to_end(&mut image_buf)?;
        let text = ocr::image_to_text(&image_buf)?;
        let expressions = Expression::from_lines(&text)?;
        let result = Expression::fold_expressions(&expressions);
        Ok(Solution {
            result
        })
    }
}

#[test]
fn can_compute_expressions() {
    let input = "+12\n\
    -10\n\
    *2";
    let exprs = Expression::from_lines(input).unwrap();
    assert_eq!(Expression::fold_expressions(&exprs), 4);
}