use nom::types::CompleteStr;

use ast::helpers::lo_name;
use ast::expression::core::Expression;
use ast::expression::variable::variable;

named!(pub access<CompleteStr, Expression>,
  do_parse!(
    var: variable >>
    accessors: many1!(
        do_parse!(
            char!('.') >>
            name: lo_name >>
            (name)
        )
    ) >>
    (Expression::Access(Box::new(var), accessors))
  )
);
