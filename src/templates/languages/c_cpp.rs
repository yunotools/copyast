//! Ignore template cho C và C++.

pub const TEMPLATE: &str = r#"

# C và C++
build/
cmake-build-*/
CMakeFiles/
CMakeCache.txt
compile_commands.json
*.o
*.obj
*.a
*.lib
*.so
*.dylib
*.dll
*.exe
*.out
*.d
"#;
