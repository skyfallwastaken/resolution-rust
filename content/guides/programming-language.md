---
title: "Building a programming language"
description: "Design a small Ruby-inspired language using LALRPOP, then write an interpreter for it."
week: 4

prize:
  title: "Rust for Rustaceans"
  description: "A really, really good book. You can also get $20 in Hetzner credits if you'd prefer."
---

Welcome to week 4! If you haven't done [week 1](/guides/getting-started), [week 2](/guides/egui-app) and [week 3](/guides/web-service), go do those first.

This week, you'll be building a tiny programming language. You'll define a grammar, parse source code into an abstract syntax tree and walk that tree to execute programs. The language we'll build together has a Ruby-ish feel to it, with `def`/`end` blocks, `if`/`then`/`end` conditionals and `puts` for output.

Here's what a program in our language looks like:

```
def greet(name)
    puts "Hello, " + name + "!";
end;

greet("world");

x = 10;
if x > 5 then
    puts "big number";
else
    puts "small number";
end;
```

We'll be using [LALRPOP](https://github.com/lalrpop/lalrpop), a parser generator for Rust. It takes a grammar file (a `.lalrpop` file describing the syntax of your language) and generates Rust code that can parse text according to those rules. You don't have to write a parser from scratch!

If you want to, you can also use a different parsing library, like `nom`, `winnow` or `chomsky`. I just prefer `lalrpop` 😄

## Using Hackatime (+ using AI)

Same as before: use [Lapse](https://lapse.hackclub.com) or the regular Hackatime plugin. Please don't use AI whilst working on this. The [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P) is a great place to ask for help!

## Getting help

- In the [#resolution-rust Slack channel](https://hackclub.enterprise.slack.com/archives/C0AFY7A312P), or
- Via email: [mahad@hackclub.com](mailto:mahad@hackclub.com)

## What is a parser generator?

When you write code in any language, the computer needs to turn your text into something it can understand. This happens in roughly two stages:

1. Parsing: turning raw text (like `x = 1 + 2`) into a structured tree that represents the meaning of the code
2. Evaluation: walking that tree and actually doing what it says

Writing a parser by hand is tedious and error-prone. A parser generator lets you describe your language's grammar in a declarative way, and it generates all the parsing code for you. You just write rules like "an expression is two things separated by a `+`" and LALRPOP handles the rest.

LALRPOP specifically generates LR(1) parsers, which means it reads input left-to-right and uses one token of lookahead to decide what to do. You don't need to understand the theory behind that, but it does mean your grammar needs to be unambiguous. For any valid input, there should only be one way to parse it.

## Setting up the project

```bash
cargo new tinylang
cd tinylang
```

Now let's add our dependencies. LALRPOP has two parts: a build-time dependency that generates Rust code from your grammar, and a runtime utility crate that the generated code depends on. Edit your `Cargo.toml` to look like this:

```toml
[package]
name = "tinylang"
version = "0.1.0"
edition = "2024"

[build-dependencies]
lalrpop = "0.23.1"

[dependencies]
lalrpop-util = { version = "0.23.1", features = ["lexer", "unicode"] }
```

The `lexer` feature gives us LALRPOP's built-in tokeniser, so we don't need to write one ourselves. The `unicode` feature adds support for Unicode in the lexer's regular expressions.

Next, create a `build.rs` file in the project root (next to `Cargo.toml`, not inside `src/`):

```rust
fn main() {
    lalrpop::process_root().unwrap();
}
```

This is a build script. Cargo runs it before compiling your code. It scans your `src/` directory for `.lalrpop` files, generates corresponding `.rs` files and puts them in Cargo's output directory. The `process_root` function is smart enough to only regenerate files when the grammar has changed.

## The abstract syntax tree

Before we write the grammar, we need to define the data structures that our parser will produce. An abstract syntax tree (AST) is a tree representation of your program's structure. Each node in the tree represents a construct in the source code.

Create `src/ast.rs`:

```rust
#[derive(Clone)]
pub enum Expr {
    Number(f64),
    StringLiteral(String),
    Bool(bool),
    Variable(String),
    BinOp(Box<Expr>, Op, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
}

#[derive(Clone)]
pub enum Statement {
    Puts(Expr),
    Assign(String, Expr),
    If(Expr, Vec<Statement>, Option<Vec<Statement>>),
    While(Expr, Vec<Statement>),
    Def(String, Vec<String>, Vec<Statement>),
    Return(Expr),
    ExprStatement(Expr),
}
```

Let's walk through what's here.

`Expr` represents anything that produces a value. It can be a literal (number, string, boolean), a variable reference, a binary operation (like `1 + 2`) or a function call. The `BinOp` variant uses `Box<Expr>` because an expression can contain other expressions, and Rust needs to know the size of an enum at compile time. A `Box` is a pointer to heap-allocated data, so its size is always the same regardless of what it points to. Without `Box`, the compiler would complain that `Expr` has infinite size (an `Expr` contains an `Expr` which contains an `Expr`...).

`Op` lists the operators we support: arithmetic (`+`, `-`, `*`, `/`), comparison (`<`, `>`) and equality (`==`).

`Statement` represents things that do something but don't necessarily produce a value. `Puts` prints an expression. `Assign` stores a value in a variable. `If` has a condition, a body and an optional else branch. `While` loops. `Def` defines a function with a name, parameters and a body. `Return` exits a function with a value. `ExprStatement` wraps an expression used as a statement (like a bare function call).

We derive `Clone` on all three enums because the interpreter will need to copy AST nodes when calling functions (saving and restoring variables).

## Writing the grammar

This is the core of the project. Create `src/tinylang.lalrpop`:

```
use crate::ast::{Expr, Op, Statement};

grammar;

match {
    r"\s*" => {},
    r"#[^\n]*" => {},
    _,
}

pub Program: Vec<Statement> = {
    <stmts:SemiStatement*> => stmts,
};

SemiStatement: Statement = {
    <s:Statement> ";" => s,
};

Statement: Statement = {
    "puts" <e:Expr> => Statement::Puts(e),
    <name:Ident> "=" <e:Expr> => Statement::Assign(name, e),
    "if" <cond:Expr> "then" <body:SemiStatement+> "end" =>
        Statement::If(cond, body, None),
    "if" <cond:Expr> "then" <body:SemiStatement+> "else" <else_body:SemiStatement+> "end" =>
        Statement::If(cond, body, Some(else_body)),
    "while" <cond:Expr> "do" <body:SemiStatement+> "end" =>
        Statement::While(cond, body),
    "def" <name:Ident> "(" <params:Comma<Ident>> ")" <body:SemiStatement+> "end" =>
        Statement::Def(name, params, body),
    "return" <e:Expr> => Statement::Return(e),
    <e:Expr> => Statement::ExprStatement(e),
};

Expr: Expr = {
    <l:Expr> "==" <r:Compare> => Expr::BinOp(Box::new(l), Op::Eq, Box::new(r)),
    <e:Compare> => e,
};

Compare: Expr = {
    <l:Compare> "<" <r:AddSub> => Expr::BinOp(Box::new(l), Op::Lt, Box::new(r)),
    <l:Compare> ">" <r:AddSub> => Expr::BinOp(Box::new(l), Op::Gt, Box::new(r)),
    <e:AddSub> => e,
};

AddSub: Expr = {
    <l:AddSub> "+" <r:MulDiv> => Expr::BinOp(Box::new(l), Op::Add, Box::new(r)),
    <l:AddSub> "-" <r:MulDiv> => Expr::BinOp(Box::new(l), Op::Sub, Box::new(r)),
    <e:MulDiv> => e,
};

MulDiv: Expr = {
    <l:MulDiv> "*" <r:Atom> => Expr::BinOp(Box::new(l), Op::Mul, Box::new(r)),
    <l:MulDiv> "/" <r:Atom> => Expr::BinOp(Box::new(l), Op::Div, Box::new(r)),
    <e:Atom> => e,
};

Atom: Expr = {
    <n:Num> => Expr::Number(n),
    <s:StringLit> => Expr::StringLiteral(s),
    "true" => Expr::Bool(true),
    "false" => Expr::Bool(false),
    <name:Ident> "(" <args:Comma<Expr>> ")" => Expr::Call(name, args),
    <name:Ident> => Expr::Variable(name),
    "(" <e:Expr> ")" => e,
};

Comma<T>: Vec<T> = {
    => vec![],
    <mut v:(<T> ",")*> <e:T> => {
        v.push(e);
        v
    }
};

Num: f64 = <s:r"[0-9]+(\.[0-9]+)?"> => s.parse().unwrap();
StringLit: String = <s:r#""[^"]*""#> => s[1..s.len()-1].to_string();
Ident: String = <s:r"[a-zA-Z_][a-zA-Z0-9_]*"> => s.to_string();
```

That's a lot of new syntax! Let's go through it section by section.

### Imports and the match block

```
use crate::ast::{Expr, Op, Statement};

grammar;

match {
    r"\s*" => {},
    r"#[^\n]*" => {},
    _,
}
```

The `use` statement at the top works just like in Rust, importing types we'll use in the action code. `grammar;` marks this file as a LALRPOP grammar.

The `match` block configures the built-in lexer. `r"\s*" => {}` tells it to skip whitespace (spaces, tabs, newlines). `r"#[^\n]*" => {}` skips comments starting with `#`. The `_` is a catch-all that accepts any other token. Without the `match` block, whitespace would be significant and our programs would need to be written on a single line!

### Statements and semicolons

```
pub Program: Vec<Statement> = {
    <stmts:SemiStatement*> => stmts,
};

SemiStatement: Statement = {
    <s:Statement> ";" => s,
};
```

A program is a list of semicolon-terminated statements. We use `SemiStatement` as a wrapper so that the parser knows where one statement ends and the next begins. Without this, the parser would get confused: is `x = foo(1)` an assignment followed by a parenthesised expression, or an assignment of a function call? The semicolons remove that ambiguity.

The `*` after `SemiStatement` means "zero or more". The `<stmts:...>` syntax binds the result to a variable called `stmts`, and the `=> stmts` part is the action code that runs when this rule matches. `pub` makes the parser's entry point publicly accessible from Rust code.

### Expression precedence

```
Expr: Expr = {
    <l:Expr> "==" <r:Compare> => Expr::BinOp(Box::new(l), Op::Eq, Box::new(r)),
    <e:Compare> => e,
};

Compare: Expr = {
    <l:Compare> "<" <r:AddSub> => ...,
    <l:Compare> ">" <r:AddSub> => ...,
    <e:AddSub> => e,
};

AddSub: Expr = {
    <l:AddSub> "+" <r:MulDiv> => ...,
    <l:AddSub> "-" <r:MulDiv> => ...,
    <e:MulDiv> => e,
};

MulDiv: Expr = {
    <l:MulDiv> "*" <r:Atom> => ...,
    <l:MulDiv> "/" <r:Atom> => ...,
    <e:Atom> => e,
};
```

This is the classic way to encode operator precedence in a grammar. Each "level" of nonterminal represents a precedence tier:

1. `Expr` handles `==` (lowest precedence)
2. `Compare` handles `<` and `>`
3. `AddSub` handles `+` and `-`
4. `MulDiv` handles `*` and `/` (highest precedence among operators)

The parser works bottom-up, so `*` and `/` bind more tightly than `+` and `-`, which bind more tightly than comparisons. This means `1 + 2 * 3` parses as `1 + (2 * 3)`, not `(1 + 2) * 3`. Each level's fallback rule (e.g. `<e:MulDiv> => e`) passes through to the next level down. This is a standard technique used in virtually every parser generator.

The left-recursive structure (`<l:AddSub> "+" <r:MulDiv>`) also gives us left-to-right evaluation. `1 - 2 - 3` parses as `(1 - 2) - 3`, not `1 - (2 - 3)`.

### Atoms and the Comma macro

```
Atom: Expr = {
    <n:Num> => Expr::Number(n),
    <s:StringLit> => Expr::StringLiteral(s),
    "true" => Expr::Bool(true),
    "false" => Expr::Bool(false),
    <name:Ident> "(" <args:Comma<Expr>> ")" => Expr::Call(name, args),
    <name:Ident> => Expr::Variable(name),
    "(" <e:Expr> ")" => e,
};
```

`Atom` represents the smallest, indivisible pieces of an expression. Notice that the function call rule (`Ident "(" Comma<Expr> ")"`) comes before the plain variable rule (`Ident`). LALRPOP tries alternatives in order, and a function call starts with an identifier just like a variable does, but it's followed by `(`. The ordering tells the parser to try the longer match first.

The `"(" <e:Expr> ")"` rule lets you group expressions with parentheses, so `(1 + 2) * 3` works as expected.

```
Comma<T>: Vec<T> = {
    => vec![],
    <mut v:(<T> ",")*> <e:T> => {
        v.push(e);
        v
    }
};
```

This is a LALRPOP macro. The `<T>` is a generic parameter, so `Comma<Expr>` produces a comma-separated list of expressions and `Comma<Ident>` produces a comma-separated list of identifiers. The first alternative (`=> vec![]`) matches an empty list. The second matches one or more items separated by commas. The `(<T> ",")*` part matches zero or more "item, comma" pairs, and the final `<e:T>` matches the last item (which doesn't have a trailing comma).

### Terminal rules

```
Num: f64 = <s:r"[0-9]+(\.[0-9]+)?"> => s.parse().unwrap();
StringLit: String = <s:r#""[^"]*""#> => s[1..s.len()-1].to_string();
Ident: String = <s:r"[a-zA-Z_][a-zA-Z0-9_]*"> => s.to_string();
```

These are the terminal rules that match raw text using regular expressions. `Num` matches integers and decimals. `StringLit` matches double-quoted strings, then strips the quotes off with `s[1..s.len()-1]`. `Ident` matches identifiers (variable and function names).

## The interpreter

Now for the fun part: actually running programs. Create `src/interpreter.rs`:

```rust
use std::collections::HashMap;
use crate::ast::{Expr, Op, Statement};

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    StringVal(String),
    Bool(bool),
    Nil,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                if *n == (*n as i64) as f64 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Value::StringVal(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

struct Function {
    params: Vec<String>,
    body: Vec<Statement>,
}

pub struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn run(&mut self, program: Vec<Statement>) -> Result<(), String> {
        for stmt in &program {
            self.exec_statement(stmt)?;
        }
        Ok(())
    }

    fn exec_statement(&mut self, stmt: &Statement) -> Result<Option<Value>, String> {
        match stmt {
            Statement::Puts(expr) => {
                let val = self.eval_expr(expr)?;
                println!("{val}");
                Ok(None)
            }
            Statement::Assign(name, expr) => {
                let val = self.eval_expr(expr)?;
                self.variables.insert(name.clone(), val);
                Ok(None)
            }
            Statement::If(cond, body, else_body) => {
                let val = self.eval_expr(cond)?;
                if self.is_truthy(&val) {
                    for s in body {
                        if let Some(ret) = self.exec_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                } else if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        if let Some(ret) = self.exec_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Ok(None)
            }
            Statement::While(cond, body) => {
                loop {
                    let val = self.eval_expr(cond)?;
                    if !self.is_truthy(&val) {
                        break;
                    }
                    for s in body {
                        if let Some(ret) = self.exec_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Ok(None)
            }
            Statement::Def(name, params, body) => {
                self.functions.insert(name.clone(), Function {
                    params: params.clone(),
                    body: body.clone(),
                });
                Ok(None)
            }
            Statement::Return(expr) => {
                let val = self.eval_expr(expr)?;
                Ok(Some(val))
            }
            Statement::ExprStatement(expr) => {
                self.eval_expr(expr)?;
                Ok(None)
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::StringLiteral(s) => Ok(Value::StringVal(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Variable(name) => {
                self.variables.get(name)
                    .cloned()
                    .ok_or_else(|| format!("undefined variable: {name}"))
            }
            Expr::BinOp(left, op, right) => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binop(l, op, r)
            }
            Expr::Call(name, args) => {
                let evaluated_args: Result<Vec<Value>, String> =
                    args.iter().map(|a| self.eval_expr(a)).collect();
                let evaluated_args = evaluated_args?;
                self.call_function(name, evaluated_args)
            }
        }
    }

    fn eval_binop(&self, left: Value, op: &Op, right: Value) -> Result<Value, String> {
        match (left, op, right) {
            (Value::Number(a), Op::Add, Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::Number(a), Op::Sub, Value::Number(b)) => Ok(Value::Number(a - b)),
            (Value::Number(a), Op::Mul, Value::Number(b)) => Ok(Value::Number(a * b)),
            (Value::Number(a), Op::Div, Value::Number(b)) => {
                if b == 0.0 {
                    Err("division by zero".to_string())
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            (Value::Number(a), Op::Eq, Value::Number(b)) => Ok(Value::Bool(a == b)),
            (Value::Number(a), Op::Lt, Value::Number(b)) => Ok(Value::Bool(a < b)),
            (Value::Number(a), Op::Gt, Value::Number(b)) => Ok(Value::Bool(a > b)),
            (Value::StringVal(a), Op::Add, Value::StringVal(b)) => {
                Ok(Value::StringVal(format!("{a}{b}")))
            }
            (Value::StringVal(a), Op::Eq, Value::StringVal(b)) => Ok(Value::Bool(a == b)),
            _ => Err("type error in binary operation".to_string()),
        }
    }

    fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        let func = self.functions.get(name)
            .ok_or_else(|| format!("undefined function: {name}"))?;

        if args.len() != func.params.len() {
            return Err(format!(
                "{name} expects {} arguments, got {}",
                func.params.len(),
                args.len()
            ));
        }

        // Save the current variables so we can restore them after the call.
        let old_vars = self.variables.clone();

        for (param, arg) in func.params.clone().into_iter().zip(args) {
            self.variables.insert(param, arg);
        }

        let body = func.body.clone();
        let mut result = Value::Nil;
        for stmt in &body {
            if let Some(ret) = self.exec_statement(stmt)? {
                result = ret;
                break;
            }
        }

        self.variables = old_vars;
        Ok(result)
    }

    fn is_truthy(&self, val: &Value) -> bool {
        match val {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::StringVal(s) => !s.is_empty(),
            Value::Nil => false,
        }
    }
}
```

This is the longest file, but it's actually quite straightforward. Let's break it down.

### Values

```rust
#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    StringVal(String),
    Bool(bool),
    Nil,
}
```

Our language has four types of runtime values. `Nil` is used as the default return value when a function doesn't explicitly return anything, similar to `nil` in Ruby or `None` in Python.

The `Display` implementation below it controls how values are printed by `puts`. The number formatting bit (`if *n == (*n as i64) as f64`) checks whether a number is a whole number, so `10.0` prints as `10` rather than `10.0`. This is purely cosmetic.

### The interpreter struct

```rust
pub struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
}
```

The interpreter keeps two hash maps: one for variables and one for function definitions. When we call a function, we save the current variables, set up the parameters and restore the old variables afterwards. This gives us basic scoping.

### Executing statements

The `exec_statement` method is a big `match` on the statement type. Most branches are straightforward, but a few things are worth calling out.

The return type `Result<Option<Value>, String>` looks a bit hairy. `Result` is for errors (like trying to use an undefined variable). `Option<Value>` is for return values from functions: `None` means "keep going" and `Some(value)` means "this function is returning a value, stop executing its body". This is how `return` works: when we hit a `Return` statement, we wrap the value in `Some` and propagate it upwards through the call stack.

The `While` loop is worth studying:

```rust
Statement::While(cond, body) => {
    loop {
        let val = self.eval_expr(cond)?;
        if !self.is_truthy(&val) {
            break;
        }
        for s in body {
            if let Some(ret) = self.exec_statement(s)? {
                return Ok(Some(ret));
            }
        }
    }
    Ok(None)
}
```

It's a Rust `loop` that re-evaluates the condition each iteration. If the condition is falsy, we break out. Otherwise we execute each statement in the body. If any statement returns a value (from a `return`), we propagate it up immediately.

### Evaluating expressions

`eval_expr` recursively evaluates an expression tree. For binary operations, it evaluates both sides first, then calls `eval_binop` to do the actual computation. The pattern match in `eval_binop` handles all the type combinations we support. Trying to add a number and a boolean, for instance, will produce a "type error" message.

String concatenation works with `+`, just like in Ruby: `"hello " + "world"` produces `"hello world"`.

### Function calls

```rust
fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, String> {
    let func = self.functions.get(name)
        .ok_or_else(|| format!("undefined function: {name}"))?;

    let old_vars = self.variables.clone();

    for (param, arg) in func.params.clone().into_iter().zip(args) {
        self.variables.insert(param, arg);
    }

    let body = func.body.clone();
    let mut result = Value::Nil;
    for stmt in &body {
        if let Some(ret) = self.exec_statement(stmt)? {
            result = ret;
            break;
        }
    }

    self.variables = old_vars;
    Ok(result)
}
```

This is worth understanding properly. We save the current variable state with `clone()`, set up the function parameters as local variables, execute the body and then restore the old state. This means functions get their own "scope", and changes to variables inside a function don't leak out to the caller.

The `.zip(args)` call pairs up parameter names with argument values. If the function is `def add(a, b)` and we call `add(1, 2)`, then `zip` produces the pairs `("a", 1)` and `("b", 2)`.

## Wiring it all together

Finally, replace `src/main.rs` with:

```rust
pub mod ast;
pub mod interpreter;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
    tinylang
);

use interpreter::Interpreter;

fn main() {
    let source = r#"
def greet(name)
    puts "Hello, " + name + "!";
end;

greet("world");

x = 10;
y = 3;
puts x + y;

def factorial(n)
    if n < 2 then
        return 1;
    end;
    return n * factorial(n - 1);
end;

result = factorial(5);
puts result;

i = 1;
while i < 6 do
    puts i;
    i = i + 1;
end;

if 10 > 5 then
    puts "ten is greater";
else
    puts "this won't print";
end;
"#;

    let parser = tinylang::ProgramParser::new();
    let program = parser.parse(source).unwrap_or_else(|e| {
        eprintln!("Parse error: {e}");
        std::process::exit(1);
    });

    let mut interp = Interpreter::new();
    if let Err(e) = interp.run(program) {
        eprintln!("Runtime error: {e}");
        std::process::exit(1);
    }
}
```

The `lalrpop_mod!` macro imports the generated parser code. The `#[allow(clippy::ptr_arg)]` and `#[rustfmt::skip]` attributes suppress lint warnings on the generated code, which we can't control.

The test program exercises most features of our language: function definitions, function calls, variables, arithmetic, recursion (the factorial function calls itself!), while loops and if/else conditionals.

## Running it

```bash
cargo run
```

The first build will take a little while because LALRPOP needs to generate the parser code. You should see:

```
Hello, world!
13
120
1
2
3
4
5
ten is greater
```

Let's trace through what happened. `greet("world")` called our function and printed a greeting. `x + y` gave us `10 + 3 = 13`. `factorial(5)` computed `5 * 4 * 3 * 2 * 1 = 120` recursively. The while loop counted from 1 to 5. And the if/else correctly took the "then" branch because `10 > 5` is true.

## What you learnt

Let's recap:

- How parser generators work: you write a grammar, LALRPOP generates the parsing code
- Abstract syntax trees: representing code as a tree of nodes that your interpreter can walk
- Operator precedence: using multiple nonterminal levels so `*` binds more tightly than `+`
- Tree-walking interpretation: recursively evaluating an AST by pattern matching on nodes
- Basic scoping: saving and restoring variables to give functions their own local state
- The `Box` type: heap-allocating data so recursive types have a known size

## Now build your own!

The tinylang example was a guided exercise. For your submission, you need to extend the language in meaningful ways and build a proper CLI for it. Your project should read source files from disk and run them.

Here's what your submission needs to include:

1. A CLI that reads a `.tl` (or whatever extension you choose) file and runs it. Check out the [`clap`](https://lib.rs/clap) library for argument parsing, or use `std::env::args` if you prefer something simpler.

2. At least two new language features. Here are some ideas, but you can come up with your own:
    - HTTP requests as a built-in, so your language can fetch data from the web. You could add a `fetch("https://...")` function that returns a string. The [`ureq`](https://lib.rs/ureq) crate makes this straightforward.
    - File I/O, so programs can read from and write to files. Perhaps `read_file("path")` and `write_file("path", "contents")`.
    - Arrays or lists with `push`, `len` and index access.
    - An `unless` keyword (the opposite of `if`, as in Ruby).
    - String interpolation, so you can write something like `"Hello, #{name}!"` instead of concatenating.
    - A `loop` construct with `break`.

3. Changes to the existing syntax. Make the language your own! Some ideas:
    - Replace semicolons with newlines as statement terminators (this is trickier than it sounds with an LR parser, but you could use a pre-processing step to insert synthetic tokens).
    - Add `elsif` support for chained conditionals.
    - Change keywords to something fun. `yell` instead of `puts`? `craft` instead of `def`?
      - If you do this one, please do something else too!
    - Add single-quoted strings as well as double-quoted ones.

Make something that you can actually use to write small scripts and, well, actually do something with :P

## Extensions

If you want to push further:

- Add line numbers to error messages so the user knows where things went wrong
- Implement closures (functions that capture variables from their enclosing scope)
- Add a REPL (read-eval-print loop) mode so you can type expressions interactively
- Write tests for your parser and interpreter using `cargo test`
- Add type coercion so you can do things like `"the answer is " + 42` without explicitly converting

## Useful resources

- [The LALRPOP book](https://lalrpop.github.io/lalrpop/) covers everything about LALRPOP grammars
- [Crafting Interpreters](https://craftinginterpreters.com) by Bob Nystrom is a brilliant free book on building programming languages
- [The `clap` crate](https://lib.rs/clap) for building CLIs (look back on Week 1 for a bit of info about how CLIs work. It doesn't use clap, but it still may be useful?)
- [The `ureq` crate](https://lib.rs/ureq) for making HTTP requests (useful for the built-in fetch feature)
  - You can use [`reqwest`](https://lib.rs/reqwest) too

## Submitting your project

Push your code to a GitHub repository. Make sure your CLI can run at least a few example programs. Double check [Hackatime](https://hackatime.hackclub.com) to make sure your time's been tracked.

See you next week! 🦀
