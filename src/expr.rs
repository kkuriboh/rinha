#[derive(Debug, PartialEq)]
pub struct Ident(String);
impl From<String> for Ident {
	fn from(value: String) -> Self {
		Self(value)
	}
}
// impl Ident {
//     #[inline]
//     pub fn val(&self) -> &str {
//         &self.0
//     }
// }

type BExpr = Box<Expr>;

#[derive(Debug, PartialEq)]
pub enum Expr {
	Int(i32),
	Bool(bool),
	Str(String),
	Variable(Ident),
	Binary {
		lhs: BExpr,
		op: BinOp,
		rhs: BExpr,
	},
	Let {
		name: Ident,
		value: BExpr,
		next: BExpr,
	},
	If {
		condition: BExpr,
		then: BExpr,
		otherwise: BExpr,
	},
	Tuple(BExpr, Box<Expr>),
	Application {
		funct: Ident,
		args: Vec<Expr>,
	},
	Abstraction {
		args: Vec<Ident>,
		body: BExpr,
	},
}

#[derive(Debug, PartialEq)]
pub enum BinOp {
	Add,
	Sub,
	Mul,
	Div,
	Rem,
	Eq,
	Neq,
	Lt,
	Gt,
	Lte,
	Gte,
	And,
	Or,
}

impl From<&str> for BinOp {
	fn from(value: &str) -> Self {
		match value {
			"Add" => Self::Add,
			"Sub" => Self::Sub,
			"Mul" => Self::Mul,
			"Div" => Self::Div,
			"Rem" => Self::Rem,
			"Eq" => Self::Eq,
			"Neq" => Self::Neq,
			"Lt" => Self::Lt,
			"Gt" => Self::Gt,
			"Lte" => Self::Lte,
			"Gte" => Self::Gte,
			"And" => Self::And,
			"Or" => Self::Or,
			_ => panic!("Invalid operator"),
		}
	}
}
