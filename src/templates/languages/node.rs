//! Ignore template cho Node.js, JavaScript và TypeScript.

pub const TEMPLATE: &str = r#"

# Node
node_modules/
.npm/
.pnpm-store/
.yarn/cache/
.yarn/unplugged/
dist/
build/
coverage/
.next/
.nuxt/
.output/
.svelte-kit/
.vite/
.turbo/
*.tsbuildinfo
npm-debug.log*
yarn-debug.log*
yarn-error.log*
pnpm-debug.log*
.env
.env.*
!.env.example
"#;
