use crate::arg_list;
use crate::MarkClosed;
use crate::Parser;

use crate::TokenKind;
use TokenKind::*;
use crate::TreeKind::*;

pub fn stmt_expr(p: &mut Parser) {
  let m = p.open();

  expr(p);
  //p.expect(Semi);

  p.close(m, StmtExpr);
}

pub fn expr(p: &mut Parser) {
  expr_rec(p, Eof);
}

fn expr_rec(p: &mut Parser, left: TokenKind) {
  let Some(mut lhs) = expr_delimited(p) else {
    return;
  };

  while p.at(LParen) {
    let m = p.open_before(lhs);
    arg_list(p);
    lhs = p.close(m, ExprCall);
  }

  loop {
    let right = p.nth(0);
    if right_binds_tighter(left, right) {
      let m = p.open_before(lhs);
      p.advance();
      expr_rec(p, right);
      lhs = p.close(m, ExprBinary);
    } else {
      break;
    }
  }
}

fn right_binds_tighter(
  left: TokenKind,
  right: TokenKind,
) -> bool {
  fn tightness(kind: TokenKind) -> Option<usize> {
    [
      // Precedence table:
      [Plus, Minus].as_slice(),
      &[Star, Slash],
    ]
    .iter()
    .position(|level| level.contains(&kind))
  }
  let Some(right_tightness) = tightness(right) else {
    return false
  };
  let Some(left_tightness) = tightness(left) else {
    assert!(left == Eof);
    return true;
  };
  right_tightness > left_tightness
}

fn expr_delimited(p: &mut Parser) -> Option<MarkClosed> {
  let result = match p.nth(0) {
    TrueKeyword | FalseKeyword | Num => {
      let m = p.open();
      p.advance();
      p.close(m, ExprLiteral)
    }
    Ident => {
      let m = p.open();
      p.advance();
      p.close(m, ExprName)
    }
    LParen => {
      let m = p.open();
      p.expect(LParen);
      expr(p);
      p.expect(RParen);
      p.close(m, ExprParen)
    }
    _ => return None,
  };
  Some(result)
}
