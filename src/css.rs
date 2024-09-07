pub struct Stylesheet {
    rules: Vec<Rule>
}

// Only supports simple selectors for now
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

pub enum Selector {
    Simple(SimpleSelector),
}

pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

pub struct Declaration {
    pub name: String,
    pub value: Value,
}

pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color)
}

pub enum Unit {
    Px,
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub type Specificity = (usize, usize, usize);

impl Selector {
    // id > class > tag, this is for solve conflicting rules
    pub fn specificity(&self) -> Specificity {
        // http://www.w3.org/TR/selectors/#specificity
        let Selector::Simple(ref simple) = *self; 
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        (a, b, c)
    }
}
