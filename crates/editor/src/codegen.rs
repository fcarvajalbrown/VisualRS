//! Pure `Graph -> Rust source` for the live panel. Never panics: any failure is
//! returned as a human-readable string the panel displays verbatim.

use vr_graph::Graph;

/// Run the full `validate -> lower -> generate` pipeline on `graph`, returning
/// formatted Rust on success or a panel-ready error string on failure
/// (validation errors joined by newlines, or a lowering/codegen error's
/// `Display`).
pub fn generate_source(graph: &Graph) -> Result<String, String> {
    graph.validate().map_err(|errs| errs.join("\n"))?;
    let program = vr_graph::lower(graph).map_err(|e| e.to_string())?;
    vr_rustgen::generate(&program).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::seed_graph;

    #[test]
    fn seed_generates_expected_rust() {
        let src = generate_source(&seed_graph()).expect("seed generates");
        assert!(src.contains("fn main"), "got:\n{src}");
        assert!(src.contains("1 + 2"), "got:\n{src}");
        assert!(src.contains("println"), "got:\n{src}");
    }
}
