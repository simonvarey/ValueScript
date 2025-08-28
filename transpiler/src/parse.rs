// valuescript_compiler\src\module_compiler.rs

use std::sync::{Arc, Mutex};

use swc_common::errors::{DiagnosticBuilder, Emitter, Handler};
use swc_common::{FileName, SourceMap};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{Syntax, TsConfig};

use valuescript_compiler::Diagnostic;

struct DiagnosticCollector {
  diagnostics: Arc<Mutex<Vec<Diagnostic>>>,
}

impl Emitter for DiagnosticCollector {
  fn emit(&mut self, db: &DiagnosticBuilder<'_>) {
    if let Some(diagnostic) = Diagnostic::from_swc(db) {
      self.diagnostics.lock().unwrap().push(diagnostic)
    }
  }
}

pub fn parse(source: &str) -> (Option<swc_ecma_ast::Program>, Vec<Diagnostic>) {
  let source_map = Arc::<SourceMap>::default();

  let diagnostics_arc = Arc::new(Mutex::new(Vec::<Diagnostic>::new()));

  let handler = Handler::with_emitter(
    true,
    false,
    Box::new(DiagnosticCollector {
      diagnostics: diagnostics_arc.clone(),
    }),
  );

  let swc_compiler = swc::Compiler::new(source_map.clone());

  let file = source_map.new_source_file(FileName::Anon, source.into());

  let result = swc_compiler.parse_js(
    file,
    &handler,
    EsVersion::Es2022,
    Syntax::Typescript(TsConfig {
      tsx: true,
      decorators: false,
      dts: false,
      no_early_errors: false,
    }),
    swc::config::IsModule::Bool(true),
    None,
  );

  let mut diagnostics = Vec::<Diagnostic>::new();
  std::mem::swap(&mut diagnostics, &mut *diagnostics_arc.lock().unwrap());

  (result.ok(), diagnostics)
}
