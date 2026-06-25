use crate::error::Result;
use std::fs;
use std::path::Path;

pub async fn handle(name: String) -> Result<()> {
    let dir = Path::new(&name);
    if dir.exists() {
        return Err(crate::error::CNodeError::Custom(format!(
            "directory {} already exists",
            name
        )));
    }

    fs::create_dir_all(dir.join("src"))?;

    let package_json = serde_json::json!({
        "name": name,
        "version": "0.1.0",
        "private": true,
        "description": "",
        "main": "index.js",
        "scripts": {
            "test": "echo \"Error: no test specified\" && exit 1",
            "start": "node index.js"
        },
        "keywords": [],
        "author": "",
        "license": "ISC"
    });
    fs::write(
        dir.join("package.json"),
        serde_json::to_string_pretty(&package_json)? + "\n",
    )?;

    fs::write(
        dir.join("index.js"),
        "console.log(\"Hello, world!\");\n",
    )?;

    fs::write(
        dir.join("README.md"),
        format!("# {}\n\n", name),
    )?;

    println!("Created project `{}`", name);
    println!("  {}/{}", dir.display(), "package.json");
    println!("  {}/{}", dir.display(), "index.js");
    println!("  {}/{}", dir.display(), "README.md");

    Ok(())
}
