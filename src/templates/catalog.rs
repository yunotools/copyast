use super::{
    default, docker,
    frameworks::{
        actix_web, angular, axum, django, electron, express, fastapi, fiber, flask, flutter, gin,
        ktor, laravel, nestjs, nextjs, nuxt, rails, react, rocket, spring_boot, svelte, sveltekit,
        symfony, tauri, vue,
    },
    languages::{
        c_cpp, csharp, dart, elixir, go, haskell, java, kotlin, lua, node, php, powershell, python,
        ruby, rust, scala, shell, sql, swift, terraform, web,
    },
};

pub struct TemplateDefinition {
    pub name: &'static str,
    pub title: &'static str,
    pub aliases: &'static [&'static str],
    pub content: &'static str,
}

const TEMPLATES: &[TemplateDefinition] = &[
    TemplateDefinition {
        name: "default",
        title: "DEFAULT",
        aliases: &["common", "general"],
        content: default::TEMPLATE,
    },
    TemplateDefinition {
        name: "rust",
        title: "RUST",
        aliases: &["rs"],
        content: rust::TEMPLATE,
    },
    TemplateDefinition {
        name: "node",
        title: "NODE / JAVASCRIPT / TYPESCRIPT",
        aliases: &["javascript", "js", "typescript", "ts", "nodejs", "node.js"],
        content: node::TEMPLATE,
    },
    TemplateDefinition {
        name: "python",
        title: "PYTHON",
        aliases: &["py"],
        content: python::TEMPLATE,
    },
    TemplateDefinition {
        name: "go",
        title: "GO",
        aliases: &["golang"],
        content: go::TEMPLATE,
    },
    TemplateDefinition {
        name: "java",
        title: "JAVA",
        aliases: &[],
        content: java::TEMPLATE,
    },
    TemplateDefinition {
        name: "kotlin",
        title: "KOTLIN",
        aliases: &["kt"],
        content: kotlin::TEMPLATE,
    },
    TemplateDefinition {
        name: "c-cpp",
        title: "C / C++",
        aliases: &["c", "cpp", "c++", "cplusplus"],
        content: c_cpp::TEMPLATE,
    },
    TemplateDefinition {
        name: "csharp",
        title: "C# / .NET",
        aliases: &["c#", "cs", "dotnet", ".net"],
        content: csharp::TEMPLATE,
    },
    TemplateDefinition {
        name: "php",
        title: "PHP",
        aliases: &[],
        content: php::TEMPLATE,
    },
    TemplateDefinition {
        name: "ruby",
        title: "RUBY",
        aliases: &["rb"],
        content: ruby::TEMPLATE,
    },
    TemplateDefinition {
        name: "swift",
        title: "SWIFT",
        aliases: &[],
        content: swift::TEMPLATE,
    },
    TemplateDefinition {
        name: "dart",
        title: "DART",
        aliases: &[],
        content: dart::TEMPLATE,
    },
    TemplateDefinition {
        name: "shell",
        title: "SHELL",
        aliases: &["sh", "bash", "zsh"],
        content: shell::TEMPLATE,
    },
    TemplateDefinition {
        name: "powershell",
        title: "POWERSHELL",
        aliases: &["pwsh", "ps1"],
        content: powershell::TEMPLATE,
    },
    TemplateDefinition {
        name: "lua",
        title: "LUA",
        aliases: &[],
        content: lua::TEMPLATE,
    },
    TemplateDefinition {
        name: "elixir",
        title: "ELIXIR",
        aliases: &["ex"],
        content: elixir::TEMPLATE,
    },
    TemplateDefinition {
        name: "haskell",
        title: "HASKELL",
        aliases: &["hs"],
        content: haskell::TEMPLATE,
    },
    TemplateDefinition {
        name: "scala",
        title: "SCALA",
        aliases: &[],
        content: scala::TEMPLATE,
    },
    TemplateDefinition {
        name: "terraform",
        title: "TERRAFORM / OPENTOFU",
        aliases: &["tf", "opentofu", "tofu"],
        content: terraform::TEMPLATE,
    },
    TemplateDefinition {
        name: "web",
        title: "HTML / CSS",
        aliases: &["html", "css", "frontend"],
        content: web::TEMPLATE,
    },
    TemplateDefinition {
        name: "sql",
        title: "SQL / DATABASE",
        aliases: &["database"],
        content: sql::TEMPLATE,
    },
    TemplateDefinition {
        name: "docker",
        title: "DOCKER",
        aliases: &["container", "dockerfile"],
        content: docker::TEMPLATE,
    },
    TemplateDefinition {
        name: "react",
        title: "REACT",
        aliases: &[],
        content: react::TEMPLATE,
    },
    TemplateDefinition {
        name: "nextjs",
        title: "NEXT.JS",
        aliases: &["next", "next.js"],
        content: nextjs::TEMPLATE,
    },
    TemplateDefinition {
        name: "vue",
        title: "VUE",
        aliases: &["vuejs", "vue.js"],
        content: vue::TEMPLATE,
    },
    TemplateDefinition {
        name: "nuxt",
        title: "NUXT",
        aliases: &["nuxtjs", "nuxt.js"],
        content: nuxt::TEMPLATE,
    },
    TemplateDefinition {
        name: "angular",
        title: "ANGULAR",
        aliases: &[],
        content: angular::TEMPLATE,
    },
    TemplateDefinition {
        name: "svelte",
        title: "SVELTE",
        aliases: &[],
        content: svelte::TEMPLATE,
    },
    TemplateDefinition {
        name: "sveltekit",
        title: "SVELTEKIT",
        aliases: &["svelte-kit", "svelte kit"],
        content: sveltekit::TEMPLATE,
    },
    TemplateDefinition {
        name: "express",
        title: "EXPRESS",
        aliases: &["expressjs", "express.js"],
        content: express::TEMPLATE,
    },
    TemplateDefinition {
        name: "nestjs",
        title: "NESTJS",
        aliases: &["nest", "nest.js", "nest-js"],
        content: nestjs::TEMPLATE,
    },
    TemplateDefinition {
        name: "electron",
        title: "ELECTRON",
        aliases: &[],
        content: electron::TEMPLATE,
    },
    TemplateDefinition {
        name: "tauri",
        title: "TAURI",
        aliases: &[],
        content: tauri::TEMPLATE,
    },
    TemplateDefinition {
        name: "django",
        title: "DJANGO",
        aliases: &[],
        content: django::TEMPLATE,
    },
    TemplateDefinition {
        name: "flask",
        title: "FLASK",
        aliases: &[],
        content: flask::TEMPLATE,
    },
    TemplateDefinition {
        name: "fastapi",
        title: "FASTAPI",
        aliases: &["fast-api", "fast api"],
        content: fastapi::TEMPLATE,
    },
    TemplateDefinition {
        name: "actix-web",
        title: "ACTIX WEB",
        aliases: &["actix", "actix_web", "actix web"],
        content: actix_web::TEMPLATE,
    },
    TemplateDefinition {
        name: "axum",
        title: "AXUM",
        aliases: &[],
        content: axum::TEMPLATE,
    },
    TemplateDefinition {
        name: "rocket",
        title: "ROCKET",
        aliases: &[],
        content: rocket::TEMPLATE,
    },
    TemplateDefinition {
        name: "gin",
        title: "GIN",
        aliases: &["gin-gonic"],
        content: gin::TEMPLATE,
    },
    TemplateDefinition {
        name: "fiber",
        title: "FIBER",
        aliases: &["gofiber"],
        content: fiber::TEMPLATE,
    },
    TemplateDefinition {
        name: "spring-boot",
        title: "SPRING BOOT",
        aliases: &["spring", "springboot", "spring boot"],
        content: spring_boot::TEMPLATE,
    },
    TemplateDefinition {
        name: "ktor",
        title: "KTOR",
        aliases: &[],
        content: ktor::TEMPLATE,
    },
    TemplateDefinition {
        name: "laravel",
        title: "LARAVEL",
        aliases: &[],
        content: laravel::TEMPLATE,
    },
    TemplateDefinition {
        name: "symfony",
        title: "SYMFONY",
        aliases: &[],
        content: symfony::TEMPLATE,
    },
    TemplateDefinition {
        name: "rails",
        title: "RUBY ON RAILS",
        aliases: &["ruby-on-rails", "ruby on rails"],
        content: rails::TEMPLATE,
    },
    TemplateDefinition {
        name: "flutter",
        title: "FLUTTER",
        aliases: &[],
        content: flutter::TEMPLATE,
    },
];

pub fn list_templates() -> &'static [TemplateDefinition] {
    TEMPLATES
}

pub fn find_template(template_name: &str) -> Option<&'static TemplateDefinition> {
    let normalized_name = template_name.trim().to_ascii_lowercase();

    TEMPLATES.iter().find(|template| {
        template.name == normalized_name || template.aliases.contains(&normalized_name.as_str())
    })
}
