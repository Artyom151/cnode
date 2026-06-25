use crate::error::Result;
use std::fs;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};

pub async fn handle(yes: bool) -> Result<()> {
    let name = if yes {
        "my-project".to_string()
    } else {
        let mut stdout = io::stdout();
        stdout.write_all(b"package name: ").await?;
        stdout.flush().await?;

        let mut stdin = io::BufReader::new(io::stdin());
        let mut input = String::new();
        stdin.read_line(&mut input).await?;
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            "my-project".to_string()
        } else {
            trimmed
        }
    };

    let description = if yes {
        String::new()
    } else {
        let mut stdout = io::stdout();
        stdout.write_all(b"description: ").await?;
        stdout.flush().await?;

        let mut stdin = io::BufReader::new(io::stdin());
        let mut input = String::new();
        stdin.read_line(&mut input).await?;
        input.trim().to_string()
    };

    let package_json = format!(
        r#"{{
  "name": "{}",
  "version": "0.0.1",
  "description": "{}",
  "main": "index.js",
  "scripts": {{
    "test": "echo \"Error: no test specified\" && exit 1"
  }},
  "keywords": [],
  "author": "",
  "license": "MIT"
}}
"#,
        name, description
    );

    fs::write("package.json", package_json)?;
    Ok(())
}
