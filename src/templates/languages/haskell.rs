//! Ignore template cho Haskell.

pub const TEMPLATE: &str = r#"

# Haskell
dist/
dist-newstyle/
.stack-work/
*.hi
*.o
*.dyn_hi
*.dyn_o
cabal.project.local
ghc.environment.*
"#;
