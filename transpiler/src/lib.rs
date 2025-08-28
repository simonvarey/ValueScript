mod parse;

use std::mem;

use parse::parse;
use swc_ecma_ast::{Expr, Module, ModuleItem, Program, Script, Stmt, VarDeclOrExpr};

/*use valuescript_compiler::{CompileResult, compile_str};
fn transpile(src: &str) -> CompileResult {
  compile_str(src)
}*/

// Helpers

fn trans_box_expr_inplace(expr: &mut Box<Expr>) {
  mem::replace(expr, Box::new(trans_expr(*expr.clone())));
}

fn trans_box_stmt_inplace(stmt: &mut Box<Stmt>) {
  mem::replace(stmt, Box::new(trans_stmt(*stmt.clone())));
}

fn trans_stmt_vec(stmts: Vec<Stmt>) -> Vec<Stmt> {
  stmts.into_iter().map(|s| trans_stmt(s)).collect()
}

// Transpile

fn trans_expr(expr: Expr) -> Expr {}

fn trans_block_stmt(mut block: swc_ecma_ast::BlockStmt) -> swc_ecma_ast::BlockStmt {
  block.stmts = trans_stmt_vec(block.stmts);
  block
}

fn trans_stmt(stmt: Stmt) -> Stmt {
  match stmt {
    Stmt::Block(mut block) => Stmt::Block(trans_block_stmt(block)),
    Stmt::Empty(_) => stmt,
    Stmt::Debugger(_) => stmt,
    Stmt::With(mut with) => {
      trans_box_expr_inplace(&mut with.obj);
      trans_box_stmt_inplace(&mut with.body);
      Stmt::With(with)
    }
    Stmt::Return(mut ret) => {
      ret.arg = ret.arg.map(|exp| Box::new(trans_expr(*exp)));
      Stmt::Return(ret)
    }
    Stmt::Labeled(mut label) => {
      trans_box_stmt_inplace(&mut label.body);
      Stmt::Labeled(label)
    }
    Stmt::Break(_) => stmt,
    Stmt::Continue(_) => stmt,
    Stmt::If(mut cond) => {
      trans_box_expr_inplace(&mut cond.test);
      trans_box_stmt_inplace(&mut cond.cons);
      Stmt::If(cond)
    }
    Stmt::Switch(mut switch) => {
      trans_box_expr_inplace(&mut switch.discriminant);
      switch.cases = switch
        .cases
        .into_iter()
        .map(|mut case| {
          case.test = case.test.map(|exp| Box::new(trans_expr(*exp)));
          case.cons = trans_stmt_vec(case.cons);
          case
        })
        .collect();
      Stmt::Switch(switch)
    }
    Stmt::Throw(mut throw) => {
      trans_box_expr_inplace(&mut throw.arg);
      Stmt::Throw(throw)
    }
    Stmt::Try(mut t) => {
      t.block = trans_block_stmt(t.block);
      t.handler = t.handler.map(|mut catch| {
        // TODO: Pattern
        catch.body = trans_block_stmt(catch.body);
        catch
      });
      t.finalizer = t.finalizer.map(|block| trans_block_stmt(block));
      Stmt::Try(t)
    }
    Stmt::While(mut w) => {
      trans_box_expr_inplace(&mut w.test);
      trans_box_stmt_inplace(&mut w.body);
      Stmt::While(w)
    }
    Stmt::DoWhile(mut dowhile) => {
      trans_box_expr_inplace(&mut dowhile.test);
      trans_box_stmt_inplace(&mut dowhile.body);
      Stmt::DoWhile(dowhile)
    }
    Stmt::For(mut f) => {
      f.init = f.init.map(|init| match init {
        VarDeclOrExpr::VarDecl(decl) => VarDeclOrExpr::VarDecl(trans_decl(decl)),
        VarDeclOrExpr::Expr(expr) => VarDeclOrExpr::Expr(Box::new(trans_expr(*expr))),
      });
      f.test = f.test.map(|exp| Box::new(trans_expr(*exp)));
      f.update = f.update.map(|exp| Box::new(trans_expr(*exp)));
      trans_box_stmt_inplace(&mut f.body);
      Stmt::For(f)
    }
    Stmt::ForIn(mut for_in) => {
      // TODO var decl/pat
      trans_box_expr_inplace(&mut for_in.right);
      trans_box_stmt_inplace(&mut for_in.body);
      Stmt::ForIn(for_in)
    }
    Stmt::ForOf(mut for_of) => {
      // TODO var decl/pat
      trans_box_expr_inplace(&mut for_of.right);
      trans_box_stmt_inplace(&mut for_of.body);
      Stmt::ForOf(for_of)
    }
    Stmt::Decl(_) => stmt, // TODO: decl
    Stmt::Expr(mut expr) => {
      trans_box_expr_inplace(&mut expr.expr);
      Stmt::Expr(expr)
    }
  }
}

fn trans_mod_item(mod_item: ModuleItem) -> ModuleItem {
  match mod_item {
    ModuleItem::ModuleDecl(_) => mod_item, //TODO decl
    ModuleItem::Stmt(stmt) => ModuleItem::Stmt(trans_stmt(stmt)),
  }
}

fn trans_script(mut script: Script) -> Script {
  script.body = trans_stmt_vec(script.body);
  script
}

fn trans_module(mut module: Module) -> Module {
  module.body = module
    .body
    .into_iter()
    .map(|mod_item| trans_mod_item(mod_item))
    .collect();
  module
}

fn trans_program(program: Program) -> Program {
  match program {
    Program::Module(module) => Program::Module(trans_module(module)),
    Program::Script(script) => Program::Script(trans_script(script)),
  }
}

fn transpile(src: &str) -> Result<Program, ()> {
  let (original_program_opt, diags) = parse(src);
  if diags.len() > 1 {
    return Err(());
  }
  let original_program = original_program_opt.ok_or(())?;
  Ok(trans_program(original_program))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parse::parse;

  #[test]
  fn scratch() {
    let compile_result = parse("export const x = 1;");
    println!("{:?}", compile_result);
  }
}
