#[cfg(any(test, feature = "rand"))]
extern crate rand;

use std::collections::HashMap;
use std::iter::Peekable;
use std::ops::Range;
use std::str::CharIndices;


struct Parser<'a> {
    text: &'a str,
    iter: Peekable<CharIndices<'a>>,
}

impl<'a> Parser<'a> {
    fn parse(&mut self) -> Result<Grammar, &'static str> {
        let mut grammar = Grammar::new();

        while let Some((term, items)) = self.parse_line()? {
            grammar.add_rule(&term, items);
        }

        Ok(grammar)
    }

    fn is_whitespace(c: char) -> bool {
        c == ' ' || c == '\t'
    }

    fn skip_whitespace(&mut self, newline: bool) {
        loop {
            if let Some(&(_, c)) = self.iter.peek() {
                if Self::is_whitespace(c) || (newline && c == '\n') {
                    self.iter.next();
                    continue;
                }
            }
            return;
        }
    }

    fn parse_line(&mut self)
        -> Result<Option<(Term, Vec<Item>)>, &'static str>
    {
        Ok(if let Some(term) = self.parse_lhs()? {
            let rule = self.parse_rule()?;
            Some((term, rule))
        } else {
            None
        })
    }

    fn parse_lhs(&mut self) -> Result<Option<Term>, &'static str> {
        self.skip_whitespace(true);
        match self.iter.next() {
            None => return Ok(None),
            Some((s, _)) => loop {
                while let Some((e, c)) = self.iter.next() {
                    if Self::is_whitespace(c) {
                        let term = Term(self.text[s..e].to_owned());
                        while let Some((_, c)) = self.iter.next() {
                            if c == '=' {
                                return Ok(Some(term));
                            } else if !Self::is_whitespace(c) {
                                return Err("Unexpected char waiting for =");
                            }
                        }
                        return Err("EOF waiting for =");
                    } else if c == '=' {
                        return Ok(Some(Term(self.text[s..e].to_owned())));
                    }
                }
                return Err("EOF reading term");
            }
        }
    }

    fn parse_rule(&mut self) -> Result<Vec<Item>, &'static str> {
        self.skip_whitespace(false);
        let mut items = Vec::new();
        loop {
            match self.iter.next() {
                None => return Ok(items),
                Some((_, '\n')) => return Ok(items),
                Some((s, '{')) => loop {
                    match self.iter.next() {
                        None => return Err("EOF reading a non-terminal"),
                        Some((e, '}')) => {
                            items.push(Item::N(Term(self.text[(s + 1)..e]
                                                    .to_owned())));
                            break;
                        }
                        Some((_, _)) => {}
                    }
                },
                Some((s, _)) => loop {
                    match self.iter.peek() {
                        None => {
                            items.push(Item::T(self.text[s..].to_owned()));
                            return Ok(items);
                        }
                        Some(&(e, '\n')) => {
                            self.iter.next();
                            items.push(Item::T(self.text[s..e].to_owned()));
                            return Ok(items);
                        }
                        Some(&(e, '{')) => {
                            items.push(Item::T(self.text[s..e].to_owned()));
                            break;
                        }
                        _ => { self.iter.next(); }
                    }
                }
            }
        }
    }
}

pub fn parse(grammar: &str) -> Result<Grammar, &'static str> {
    let mut parser = Parser {
        text: grammar,
        iter: grammar.char_indices().peekable(),
    };

    parser.parse()
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Term(pub String);

pub enum Item {
    T(String),
    N(Term),
}

impl<S: AsRef<str>> From<S> for Item {
    fn from(text: S) -> Item {
        Item::T(text.as_ref().to_owned())
    }
}

impl<'a> From<&'a Term> for Item {
    fn from(term: &'a Term) -> Item {
        Item::N((*term).clone())
    }
}

pub struct Grammar {
    rules: HashMap<Term, Vec<Vec<Item>>>,
}

pub trait Chooser {
    fn choose(&mut self, range: Range<usize>) -> usize;
}

impl Grammar {
    pub fn new() -> Grammar {
        Grammar { rules: HashMap::new() }
    }

    pub fn add_rule(&mut self, term: &Term, items: Vec<Item>) {
        if let Some(rules) = self.rules.get_mut(term) {
            rules.push(items);
            return;
        }
        self.rules.insert(term.clone(), vec![items]);
    }

    pub fn generate<C: Chooser>(&self, target: &Term, chooser: &mut C)
        -> String
    {
        let mut result = String::new();
        self.generate_rec(&mut result, target, chooser);
        result
    }

    fn generate_rec<C: Chooser>(&self, result: &mut String,
                                target: &Term, chooser: &mut C) {
        let recipes = self.rules.get(target).unwrap();
        let recipe = if recipes.len() == 1 {
            &recipes[0]
        } else {
            &recipes[chooser.choose(0..recipes.len())]
        };
        for item in recipe {
            match item {
                &Item::T(ref s) => result.push_str(s),
                &Item::N(ref t) => self.generate_rec(result, t, chooser),
            }
        }
    }

    #[cfg(feature = "rand")]
    pub fn generate_random(&self, target: &Term) -> String {
        let mut rng = rand::thread_rng();
        self.generate(target, &mut rng)
    }
}

macro_rules! terms {
    ( $( $n : ident ),* ) => {
        terms!( $( $n, )* )
    };
    ( $( $n : ident, )* ) => {
        $(
            let $n = Term(stringify!($n).to_owned());
        )*
    };
}

macro_rules! vecfrom {
    ( $( $x : expr ),* ) => {
        vecfrom!( $( $x, )* )
    };
    ( $( $x : expr, )* ) => {
        vec![ $( From::from( $x ), )* ]
    };
}

#[cfg(feature = "rand")]
impl<R: rand::Rng> Chooser for R {
    fn choose(&mut self, range: Range<usize>) -> usize {
        self.gen_range(range.start, range.end)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use ::Chooser;
    use ::Grammar;
    use ::Term;
    use ::parse;

    struct MockChooser<'a>(&'a [usize]);

    impl<'a> Chooser for MockChooser<'a> {
        fn choose(&mut self, _range: Range<usize>) -> usize {
            if self.0.is_empty() {
                panic!("Ran out of choices in MockChooser");
            } else {
                let out = self.0[0];
                self.0 = &self.0[1..];
                out
            }
        }
    }

    #[test]
    fn test_generate() {
        terms!(sentence, who, drink);
        let mut grammar = Grammar::new();
        grammar.add_rule(&sentence, vecfrom![&who, " drinks ", &drink]);
        grammar.add_rule(&who, vecfrom!["the ", "cat"]);
        grammar.add_rule(&drink, vecfrom!["milk"]);
        grammar.add_rule(&drink, vecfrom!["water"]);

        assert_eq!(grammar.generate(&sentence, &mut MockChooser(&[0])),
                   "the cat drinks milk");
        assert_eq!(grammar.generate(&sentence, &mut MockChooser(&[1])),
                   "the cat drinks water");
    }
    #[test]
    fn test_parse_generate() {
        let grammar = parse(
            "sentence = {who} waited {howlong} for your {adjective} {noun} {start}
             who = {things}
             who = {number} {things}
             number = a thousand
             number = millions of
             number = countless
             things = alien lights
             things = people
             things = martians
             things = country leaders
             howlong = light years
             howlong = eons
             howlong = many moons
             howlong = an eternity
             howlong = a hundred years
             adjective = sweet
             adjective = wonderful
             adjective = awesome
             adjective = magnificent
             adjective = confident
             noun = armpits
             noun = lips
             noun = toes
             start = to come into this world
             start = to appear
             start = to exist").expect("Parsing error");

        assert_eq!(grammar.generate(&Term("sentence".to_owned()),
                                    &mut MockChooser(&[1, 0, 0, 4, 3, 0, 2])),
                   "a thousand alien lights waited a hundred years for your \
                    magnificent armpits to exist");
    }
}
