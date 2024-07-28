use crate::{Parser, TokenKind, TokenKind::*, TreeKind::*};

const PARAM_LIST_RECOVERY: &[TokenKind] = &[DefKeyword, LCurly, LParen];
pub fn type_param(p: &mut Parser) {
    assert!(p.at(LSquare));
    let m = p.open();

    p.expect(LSquare);
    while !p.at(RSquare) && !p.eof() {
        if p.at(Ident) {
            param(p);
        } else {
            if p.at_any(PARAM_LIST_RECOVERY) {
                break;
            }
            p.advance_with_error(crate::ErrKind::TypeExpr);
        }
    }
    p.expect(RSquare);

    p.close(m, ParamList);
}

fn param(p: &mut Parser) {
    assert!(p.at(Ident));
    let m = p.open();

    p.expect(Ident);
    p.expect(Colon);
    type_expr(p);
    if !p.at(RSquare) {
        p.expect(Comma);
    }

    p.close(m, Param);
}

pub fn type_expr(p: &mut Parser) {
    let m = p.open();
    if p.at(LParen) {
        p.expect(LParen);
        while !p.at(RParen) && !p.eof() {
            if p.at(Ident) || p.at(LParen) {
                type_expr(p);
                if !p.at(RParen) {
                    p.expect(Comma);
                }
            } else {
                if p.at_any(PARAM_LIST_RECOVERY) {
                    break;
                }
                p.advance_with_error(crate::ErrKind::TypeExpr);
            }
        }
        p.expect(RParen);
    } else {
        p.expect(Ident);
        if p.at(Arrow) {
            p.expect(Arrow);
            type_expr(p);
        }
    }
    p.close(m, TypeExpr);
}
