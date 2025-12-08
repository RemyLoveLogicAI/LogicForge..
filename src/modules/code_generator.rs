//! Code Generator - Generates project scaffolds and code

use crate::models::{ProjectSpec, TemplateType};
use crate::utils::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

/// Code generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    pub files: Vec<GeneratedFile>,
    pub dependencies: Vec<Dependency>,
    pub scripts: HashMap<String, String>,
    pub readme_content: String,
}

/// A generated file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
    pub is_binary: bool,
}

/// A project dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub is_dev: bool,
}

/// Code generator configuration
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub include_tests: bool,
    pub include_ci: bool,
    pub include_docker: bool,
    pub typescript: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            include_tests: true,
            include_ci: true,
            include_docker: true,
            typescript: true,
        }
    }
}

/// Code generator for project scaffolding
pub struct CodeGenerator {
    config: GeneratorConfig,
}

impl CodeGenerator {
    /// Create a new code generator
    pub fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default_generator() -> Self {
        Self::new(GeneratorConfig::default())
    }

    /// Generate code for a project specification
    pub fn generate(&self, spec: &ProjectSpec) -> Result<GeneratedCode> {
        info!("Generating code for: {} ({})", spec.name, spec.template_type);

        match spec.template_type {
            TemplateType::MicroSaas => self.generate_micro_saas(spec),
            TemplateType::ApiService => self.generate_api_service(spec),
            TemplateType::ContentSite => self.generate_content_site(spec),
            TemplateType::ChromeExtension => self.generate_chrome_extension(spec),
            TemplateType::CliTool => self.generate_cli_tool(spec),
            TemplateType::MobileApp => self.generate_mobile_app(spec),
            TemplateType::Custom(_) => self.generate_basic_project(spec),
        }
    }

    /// Generate a micro-SaaS project
    fn generate_micro_saas(&self, spec: &ProjectSpec) -> Result<GeneratedCode> {
        let mut files = Vec::new();
        let mut dependencies = Vec::new();

        // Package.json
        files.push(GeneratedFile {
            path: "package.json".to_string(),
            content: self.generate_package_json(spec, &[
                ("next", "14.0.4"),
                ("react", "18.2.0"),
                ("react-dom", "18.2.0"),
                ("tailwindcss", "3.4.0"),
            ]),
            is_binary: false,
        });

        // TypeScript config
        if self.config.typescript {
            files.push(GeneratedFile {
                path: "tsconfig.json".to_string(),
                content: self.generate_tsconfig(),
                is_binary: false,
            });
        }

        // Next.js config
        files.push(GeneratedFile {
            path: "next.config.js".to_string(),
            content: r#"/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
}

module.exports = nextConfig
"#.to_string(),
            is_binary: false,
        });

        // Tailwind config
        files.push(GeneratedFile {
            path: "tailwind.config.js".to_string(),
            content: self.generate_tailwind_config(),
            is_binary: false,
        });

        // Main page
        let ext = if self.config.typescript { "tsx" } else { "jsx" };
        files.push(GeneratedFile {
            path: format!("src/app/page.{}", ext),
            content: self.generate_nextjs_page(spec),
            is_binary: false,
        });

        // Layout
        files.push(GeneratedFile {
            path: format!("src/app/layout.{}", ext),
            content: self.generate_nextjs_layout(spec),
            is_binary: false,
        });

        // Global CSS
        files.push(GeneratedFile {
            path: "src/app/globals.css".to_string(),
            content: r#"@tailwind base;
@tailwind components;
@tailwind utilities;
"#.to_string(),
            is_binary: false,
        });

        // Environment template
        files.push(GeneratedFile {
            path: ".env.example".to_string(),
            content: self.generate_env_example(spec),
            is_binary: false,
        });

        // Git ignore
        files.push(GeneratedFile {
            path: ".gitignore".to_string(),
            content: self.generate_gitignore("node"),
            is_binary: false,
        });

        // Add CI if configured
        if self.config.include_ci {
            files.push(GeneratedFile {
                path: ".github/workflows/ci.yml".to_string(),
                content: self.generate_node_ci(),
                is_binary: false,
            });
        }

        // Add Docker if configured
        if self.config.include_docker {
            files.push(GeneratedFile {
                path: "Dockerfile".to_string(),
                content: self.generate_nextjs_dockerfile(),
                is_binary: false,
            });
        }

        // Dependencies
        dependencies.push(Dependency {
            name: "next".to_string(),
            version: "14.0.4".to_string(),
            is_dev: false,
        });
        dependencies.push(Dependency {
            name: "react".to_string(),
            version: "18.2.0".to_string(),
            is_dev: false,
        });
        dependencies.push(Dependency {
            name: "typescript".to_string(),
            version: "5.3.3".to_string(),
            is_dev: true,
        });

        let scripts = HashMap::from([
            ("dev".to_string(), "next dev".to_string()),
            ("build".to_string(), "next build".to_string()),
            ("start".to_string(), "next start".to_string()),
            ("lint".to_string(), "next lint".to_string()),
        ]);

        Ok(GeneratedCode {
            files,
            dependencies,
            scripts,
            readme_content: self.generate_readme(spec),
        })
    }

    /// Generate an API service project
    fn generate_api_service(&self, spec: &ProjectSpec) -> Result<GeneratedCode> {
        let mut files = Vec::new();

        // Package.json
        files.push(GeneratedFile {
            path: "package.json".to_string(),
            content: self.generate_package_json(spec, &[
                ("express", "4.18.2"),
                ("cors", "2.8.5"),
                ("helmet", "7.1.0"),
            ]),
            is_binary: false,
        });

        // Main server file
        let ext = if self.config.typescript { "ts" } else { "js" };
        files.push(GeneratedFile {
            path: format!("src/index.{}", ext),
            content: self.generate_express_server(spec),
            is_binary: false,
        });

        // Routes
        files.push(GeneratedFile {
            path: format!("src/routes/index.{}", ext),
            content: self.generate_express_routes(spec),
            is_binary: false,
        });

        // TypeScript config
        if self.config.typescript {
            files.push(GeneratedFile {
                path: "tsconfig.json".to_string(),
                content: self.generate_tsconfig(),
                is_binary: false,
            });
        }

        // Git ignore
        files.push(GeneratedFile {
            path: ".gitignore".to_string(),
            content: self.generate_gitignore("node"),
            is_binary: false,
        });

        // Environment template
        files.push(GeneratedFile {
            path: ".env.example".to_string(),
            content: self.generate_env_example(spec),
            is_binary: false,
        });

        // Add CI
        if self.config.include_ci {
            files.push(GeneratedFile {
                path: ".github/workflows/ci.yml".to_string(),
                content: self.generate_node_ci(),
                is_binary: false,
            });
        }

        let scripts = HashMap::from([
            ("dev".to_string(), "ts-node src/index.ts".to_string()),
            ("build".to_string(), "tsc".to_string()),
            ("start".to_string(), "node dist/index.js".to_string()),
        ]);

        Ok(GeneratedCode {
            files,
            dependencies: vec![],
            scripts,
            readme_content: self.generate_readme(spec),
        })
    }

    /// Generate a content site project
    fn generate_content_site(&self, spec: &ProjectSpec) -> Result<GeneratedCode> {
        let files = vec![
            // Package.json for Astro
            GeneratedFile {
                path: "package.json".to_string(),
                content: self.generate_package_json(spec, &[
                    ("astro", "4.0.0"),
                    ("@astrojs/tailwind", "5.1.0"),
                ]),
                is_binary: false,
            },
            // Astro config
            GeneratedFile {
                path: "astro.config.mjs".to_string(),
                content: r#"import { defineConfig } from 'astro/config';
import tailwind from '@astrojs/tailwind';

export default defineConfig({
  integrations: [tailwind()],
});
"#.to_string(),
                is_binary: false,
            },
            // Main page
            GeneratedFile {
                path: "src/pages/index.astro".to_string(),
                content: self.generate_astro_page(spec),
                is_binary: false,
            },
            // Layout
            GeneratedFile {
                path: "src/layouts/Layout.astro".to_string(),
                content: self.generate_astro_layout(spec),
                is_binary: false,
            },
            // Git ignore
            GeneratedFile {
                path: ".gitignore".to_string(),
                content: self.generate_gitignore("node"),
                is_binary: false,
            },
        ];

        let scripts = HashMap::from([
            ("dev".to_string(), "astro dev".to_string()),
            ("build".to_string(), "astro build".to_string()),
            ("preview".to_string(), "astro preview".to_string()),
        ]);

        Ok(GeneratedCode {
            files,
            dependencies: vec![],
            scripts,
            readme_content: self.generate_readme(spec),
        })
    }

    /// Generate a Chrome extension project
    fn generate_chrome_extension(&self, spec: &ProjectSpec) -> Result<GeneratedCode> {
        let files = vec![
            // Manifest
            GeneratedFile {
                path: "manifest.json".to_string(),
                content: self.generate_chrome_manifest(spec),
                is_binary: false,
            },
            // Background script
            GeneratedFile {
                path: "src/background.ts".to_string(),
                content: r#"chrome.runtime.onInstalled.addListener(() => {
  console.log('Extension installed');
});
"#.to_string(),
                is_binary: false,
            },
            // Popup HTML
            GeneratedFile {
                path: "src/popup/popup.html".to_string(),
                content: self.generate_popup_html(spec),
                is_binary: false,
            },
            // Package.json
            GeneratedFile {
                path: "package.json".to_string(),
                content: self.generate_package_json(spec, &[
                    ("typescript", "5.3.3"),
                    ("webpack", "5.89.0"),
                ]),
                is_binary: false,
            },
        ];

        let scripts = HashMap::from([
            ("build".to_string(), "webpack --mode production".to_string()),
            ("dev".to_string(), "webpack --mode development --watch".to_string()),
        ]);

        Ok(GeneratedCode {
            files,
            dependencies: vec![],
            scripts,
            readme_content: self.generate_readme(spec),
        })
    }

    /// Generate a CLI tool project (Rust)
    fn generate_cli_tool(&self, spec: &ProjectSpec) -> Result<GeneratedCode> {
        let mut files = Vec::new();

        // Cargo.toml
        files.push(GeneratedFile {
            path: "Cargo.toml".to_string(),
            content: self.generate_cargo_toml(spec),
            is_binary: false,
        });

        // Main.rs
        files.push(GeneratedFile {
            path: "src/main.rs".to_string(),
            content: self.generate_rust_main(spec),
            is_binary: false,
        });

        // Git ignore
        files.push(GeneratedFile {
            path: ".gitignore".to_string(),
            content: self.generate_gitignore("rust"),
            is_binary: false,
        });

        // CI for Rust
        if self.config.include_ci {
            files.push(GeneratedFile {
                path: ".github/workflows/ci.yml".to_string(),
                content: self.generate_rust_ci(),
                is_binary: false,
            });
        }

        let scripts = HashMap::new();

        Ok(GeneratedCode {
            files,
            dependencies: vec![],
            scripts,
            readme_content: self.generate_readme(spec),
        })
    }

    /// Generate a mobile app project
    fn generate_mobile_app(&self, spec: &ProjectSpec) -> Result<GeneratedCode> {
        let files = vec![
            // Package.json for React Native / Expo
            GeneratedFile {
                path: "package.json".to_string(),
                content: self.generate_package_json(spec, &[
                    ("expo", "49.0.0"),
                    ("react-native", "0.72.6"),
                ]),
                is_binary: false,
            },
            // App entry
            GeneratedFile {
                path: "App.tsx".to_string(),
                content: self.generate_expo_app(spec),
                is_binary: false,
            },
            // Babel config
            GeneratedFile {
                path: "babel.config.js".to_string(),
                content: r#"module.exports = function(api) {
  api.cache(true);
  return {
    presets: ['babel-preset-expo'],
  };
};
"#.to_string(),
                is_binary: false,
            },
        ];

        let scripts = HashMap::from([
            ("start".to_string(), "expo start".to_string()),
            ("android".to_string(), "expo start --android".to_string()),
            ("ios".to_string(), "expo start --ios".to_string()),
        ]);

        Ok(GeneratedCode {
            files,
            dependencies: vec![],
            scripts,
            readme_content: self.generate_readme(spec),
        })
    }

    /// Generate a basic project
    fn generate_basic_project(&self, spec: &ProjectSpec) -> Result<GeneratedCode> {
        let files = vec![
            GeneratedFile {
                path: "README.md".to_string(),
                content: self.generate_readme(spec),
                is_binary: false,
            },
            GeneratedFile {
                path: ".gitignore".to_string(),
                content: self.generate_gitignore("node"),
                is_binary: false,
            },
        ];

        Ok(GeneratedCode {
            files,
            dependencies: vec![],
            scripts: HashMap::new(),
            readme_content: self.generate_readme(spec),
        })
    }

    // Helper methods for generating specific files

    fn generate_package_json(&self, spec: &ProjectSpec, deps: &[(&str, &str)]) -> String {
        let deps_str: Vec<String> = deps
            .iter()
            .map(|(name, version)| format!("    \"{}\": \"^{}\"", name, version))
            .collect();

        format!(
            r#"{{
  "name": "{}",
  "version": "0.1.0",
  "description": "{}",
  "scripts": {{
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "eslint ."
  }},
  "dependencies": {{
{}
  }},
  "devDependencies": {{
    "typescript": "^5.3.3",
    "@types/node": "^20.10.0",
    "@types/react": "^18.2.0"
  }}
}}
"#,
            spec.name.to_lowercase().replace(' ', "-"),
            spec.description,
            deps_str.join(",\n")
        )
    }

    fn generate_tsconfig(&self) -> String {
        r#"{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["dom", "dom.iterable", "esnext"],
    "allowJs": true,
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve",
    "incremental": true,
    "plugins": [{ "name": "next" }],
    "paths": { "@/*": ["./src/*"] }
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
"#.to_string()
    }

    fn generate_tailwind_config(&self) -> String {
        r#"/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
"#.to_string()
    }

    fn generate_nextjs_page(&self, spec: &ProjectSpec) -> String {
        format!(
            r#"export default function Home() {{
  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-24">
      <h1 className="text-4xl font-bold mb-4">{}</h1>
      <p className="text-xl text-gray-600">{}</p>
    </main>
  );
}}
"#,
            spec.name, spec.description
        )
    }

    fn generate_nextjs_layout(&self, spec: &ProjectSpec) -> String {
        format!(
            r#"import './globals.css'
import type {{ Metadata }} from 'next'

export const metadata: Metadata = {{
  title: '{}',
  description: '{}',
}}

export default function RootLayout({{
  children,
}}: {{
  children: React.ReactNode
}}) {{
  return (
    <html lang="en">
      <body>{{children}}</body>
    </html>
  )
}}
"#,
            spec.name, spec.description
        )
    }

    fn generate_express_server(&self, spec: &ProjectSpec) -> String {
        format!(
            r#"import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import routes from './routes';

const app = express();
const PORT = process.env.PORT || 3000;

// Middleware
app.use(helmet());
app.use(cors());
app.use(express.json());

// Routes
app.use('/api', routes);

// Health check
app.get('/health', (req, res) => {{
  res.json({{ status: 'ok', service: '{}' }});
}});

app.listen(PORT, () => {{
  console.log(`Server running on port ${{PORT}}`);
}});

export default app;
"#,
            spec.name
        )
    }

    fn generate_express_routes(&self, _spec: &ProjectSpec) -> String {
        r#"import { Router } from 'express';

const router = Router();

router.get('/', (req, res) => {
  res.json({ message: 'Welcome to the API' });
});

export default router;
"#.to_string()
    }

    fn generate_astro_page(&self, spec: &ProjectSpec) -> String {
        format!(
            r#"---
import Layout from '../layouts/Layout.astro';
---

<Layout title="{}">
  <main class="container mx-auto px-4 py-8">
    <h1 class="text-4xl font-bold mb-4">{}</h1>
    <p class="text-xl text-gray-600">{}</p>
  </main>
</Layout>
"#,
            spec.name, spec.name, spec.description
        )
    }

    fn generate_astro_layout(&self, spec: &ProjectSpec) -> String {
        format!(
            r#"---
interface Props {{
  title: string;
}}

const {{ title }} = Astro.props;
---

<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width" />
    <title>{{title}} | {}</title>
  </head>
  <body>
    <slot />
  </body>
</html>
"#,
            spec.name
        )
    }

    fn generate_chrome_manifest(&self, spec: &ProjectSpec) -> String {
        format!(
            r#"{{
  "manifest_version": 3,
  "name": "{}",
  "version": "1.0.0",
  "description": "{}",
  "permissions": ["storage"],
  "action": {{
    "default_popup": "src/popup/popup.html",
    "default_title": "{}"
  }},
  "background": {{
    "service_worker": "src/background.js"
  }}
}}
"#,
            spec.name, spec.description, spec.name
        )
    }

    fn generate_popup_html(&self, spec: &ProjectSpec) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
  <title>{}</title>
  <style>
    body {{ width: 300px; padding: 16px; font-family: system-ui; }}
    h1 {{ font-size: 18px; margin-bottom: 8px; }}
  </style>
</head>
<body>
  <h1>{}</h1>
  <p>{}</p>
</body>
</html>
"#,
            spec.name, spec.name, spec.description
        )
    }

    fn generate_cargo_toml(&self, spec: &ProjectSpec) -> String {
        format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
description = "{}"

[dependencies]
clap = {{ version = "4", features = ["derive"] }}
anyhow = "1"
"#,
            spec.name.to_lowercase().replace(' ', "-"),
            spec.description
        )
    }

    fn generate_rust_main(&self, spec: &ProjectSpec) -> String {
        format!(
            r#"use clap::Parser;

#[derive(Parser)]
#[command(name = "{}")]
#[command(about = "{}")]
struct Cli {{
    #[arg(short, long)]
    verbose: bool,
}}

fn main() -> anyhow::Result<()> {{
    let cli = Cli::parse();

    if cli.verbose {{
        println!("Running in verbose mode");
    }}

    println!("Hello from {}!");
    Ok(())
}}
"#,
            spec.name.to_lowercase().replace(' ', "-"),
            spec.description,
            spec.name
        )
    }

    fn generate_expo_app(&self, spec: &ProjectSpec) -> String {
        format!(
            r#"import {{ StatusBar }} from 'expo-status-bar';
import {{ StyleSheet, Text, View }} from 'react-native';

export default function App() {{
  return (
    <View style={{styles.container}}>
      <Text style={{styles.title}}>{}</Text>
      <Text style={{styles.description}}>{}</Text>
      <StatusBar style="auto" />
    </View>
  );
}}

const styles = StyleSheet.create({{
  container: {{
    flex: 1,
    backgroundColor: '#fff',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  }},
  title: {{
    fontSize: 24,
    fontWeight: 'bold',
    marginBottom: 10,
  }},
  description: {{
    fontSize: 16,
    color: '#666',
    textAlign: 'center',
  }},
}});
"#,
            spec.name, spec.description
        )
    }

    fn generate_gitignore(&self, lang: &str) -> String {
        match lang {
            "rust" => r#"target/
Cargo.lock
*.pdb
"#.to_string(),
            _ => r#"node_modules/
.next/
dist/
.env
.env.local
*.log
.DS_Store
"#.to_string(),
        }
    }

    fn generate_env_example(&self, spec: &ProjectSpec) -> String {
        format!(
            r#"# {} Environment Variables
NODE_ENV=development
PORT=3000
DATABASE_URL=
API_KEY=
"#,
            spec.name
        )
    }

    fn generate_node_ci(&self) -> String {
        r#"name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      - run: npm ci
      - run: npm test
      - run: npm run build
"#.to_string()
    }

    fn generate_rust_ci(&self) -> String {
        r#"name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
"#.to_string()
    }

    fn generate_nextjs_dockerfile(&self) -> String {
        r#"FROM node:20-alpine AS base

FROM base AS deps
WORKDIR /app
COPY package*.json ./
RUN npm ci

FROM base AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN npm run build

FROM base AS runner
WORKDIR /app
ENV NODE_ENV production
COPY --from=builder /app/public ./public
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
EXPOSE 3000
CMD ["node", "server.js"]
"#.to_string()
    }

    fn generate_readme(&self, spec: &ProjectSpec) -> String {
        format!(
            r#"# {}

{}

## Features

{}

## Tech Stack

{}

## Getting Started

### Prerequisites

- Node.js 20+
- npm or yarn

### Installation

```bash
npm install
```

### Development

```bash
npm run dev
```

### Build

```bash
npm run build
```

## License

MIT
"#,
            spec.name,
            spec.description,
            spec.features
                .iter()
                .map(|f| format!("- {}", f))
                .collect::<Vec<_>>()
                .join("\n"),
            spec.tech_stack.join(", ")
        )
    }

    /// Write generated code to filesystem
    pub fn write_to_disk(&self, code: &GeneratedCode, base_path: &Path) -> Result<()> {
        for file in &code.files {
            let file_path = base_path.join(&file.path);

            // Create parent directories
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&file_path, &file.content)?;
            info!("Created: {}", file.path);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_spec() -> ProjectSpec {
        ProjectSpec {
            name: "Test Project".to_string(),
            description: "A test project".to_string(),
            template_type: TemplateType::MicroSaas,
            features: vec!["Auth".to_string()],
            tech_stack: vec!["Next.js".to_string()],
            monetization: None,
            target_users: None,
            additional_requirements: vec![],
        }
    }

    #[test]
    fn test_code_generator_creation() {
        let generator = CodeGenerator::default_generator();
        assert!(generator.config.include_tests);
    }

    #[test]
    fn test_generate_micro_saas() {
        let generator = CodeGenerator::default_generator();
        let spec = create_test_spec();

        let result = generator.generate(&spec);
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(!code.files.is_empty());
        assert!(code.files.iter().any(|f| f.path == "package.json"));
    }

    #[test]
    fn test_generate_api_service() {
        let generator = CodeGenerator::default_generator();
        let mut spec = create_test_spec();
        spec.template_type = TemplateType::ApiService;

        let result = generator.generate(&spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_cli_tool() {
        let generator = CodeGenerator::default_generator();
        let mut spec = create_test_spec();
        spec.template_type = TemplateType::CliTool;

        let result = generator.generate(&spec);
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.files.iter().any(|f| f.path == "Cargo.toml"));
    }
}
