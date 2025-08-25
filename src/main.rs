use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use dirs::home_dir;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use tokio::fs as tokio_fs;

#[derive(Parser)]
#[command(name = "tl")]
#[command(about = "A tool installer and manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install { tool_name: String },
    Uninstall { tool_name: String },
    List,
}

#[derive(Serialize, Deserialize)]
struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Tool {
    name: String,
    description: String,
    github_repo: String,
    install_method: String,
    binary_name: String,
    install_path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { tool_name } => {
            install_tool(&tool_name).await?;
        }
        Commands::Uninstall { tool_name } => {
            uninstall_tool(&tool_name).await?;
        }
        Commands::List => {
            list_tools().await?;
        }
    }

    Ok(())
}

async fn load_tool_registry() -> Result<ToolRegistry> {
    let registry_path = get_registry_path()?;
    let content = tokio_fs::read_to_string(registry_path).await?;
    let registry: ToolRegistry = serde_json::from_str(&content)?;
    Ok(registry)
}

fn get_registry_path() -> Result<PathBuf> {
    // First try current directory
    let current_dir = std::env::current_dir()?;
    let registry_path = current_dir.join("tools.json");
    if registry_path.exists() {
        return Ok(registry_path);
    }
    
    // Then try relative to executable
    let mut path = std::env::current_exe()?;
    path.pop();
    path.pop();
    path.push("tools.json");
    if path.exists() {
        return Ok(path);
    }
    
    Err(anyhow!("Registry file not found. Looked for tools.json in current directory and project root."))
}

async fn install_tool(tool_name: &str) -> Result<()> {
    let registry = load_tool_registry().await?;
    
    let tool = registry.tools.get(tool_name)
        .ok_or_else(|| anyhow!("Tool '{}' not found in registry", tool_name))?;

    // Check if tool is already installed
    let install_path = expand_install_path(&tool.install_path)?;
    let binary_path = install_path.join(&tool.binary_name);
    
    if binary_path.exists() {
        println!("⚠️  {} is already installed at {}", tool.name, binary_path.display());
        print!("Do you want to override it? [y/N]: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        if input != "y" && input != "yes" {
            println!("Installation cancelled.");
            return Ok(());
        }
        println!();
    }

    println!("📦 Installing {}...", tool.name);
    println!("   {}", tool.description);

    match tool.install_method.as_str() {
        "github_release" => {
            install_from_github_release(tool).await?;
        }
        _ => {
            return Err(anyhow!("Unsupported install method: {}", tool.install_method));
        }
    }

    Ok(())
}

async fn install_from_github_release(tool: &Tool) -> Result<()> {
    let client = Client::new();
    let api_url = format!("https://api.github.com/repos/{}/releases/latest", tool.github_repo);
    
    println!("🔍 Fetching latest release information...");
    let response = client
        .get(&api_url)
        .header("User-Agent", "tl-tool-installer")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch release info: {}", response.status()));
    }

    let release: serde_json::Value = response.json().await?;
    let assets = release["assets"].as_array()
        .ok_or_else(|| anyhow!("No assets found in release"))?;

    // Find the appropriate asset for the current platform
    let platform = get_platform_string();
    let asset = find_platform_asset(assets, &platform)?;
    
    let download_url = asset["browser_download_url"].as_str()
        .ok_or_else(|| anyhow!("No download URL found"))?;

    println!("⬇️  Downloading binary...");
    
    // Download the binary
    let response = client.get(download_url).send().await?;
    let bytes = response.bytes().await?;

    // Determine install path
    let install_path = expand_install_path(&tool.install_path)?;
    fs::create_dir_all(&install_path)?;
    
    let binary_path = install_path.join(&tool.binary_name);
    
    // Write the binary
    fs::write(&binary_path, bytes)?;
    
    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)?;
    }

    println!("✅ Successfully installed {} to {}", tool.name, binary_path.display());
    
    // Check if the install path is in PATH
    if !is_in_path(&install_path)? {
        println!();
        println!("⚠️  {} is not in your PATH", install_path.display());
        println!("To use {} from anywhere, add this directory to your PATH:", tool.name);
        println!("  echo 'export PATH=\"$PATH:{}\"' >> ~/.bashrc", install_path.display());
        println!("  source ~/.bashrc");
        println!("Or run this once to add it:");
        println!("  export PATH=\"$PATH:{}\"", install_path.display());
    }

    Ok(())
}

fn get_platform_string() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    
    match (os, arch) {
        ("linux", "x86_64") => "linux-amd64".to_string(),
        ("linux", "aarch64") => "linux-arm64".to_string(),
        ("macos", "x86_64") => "darwin-amd64".to_string(),
        ("macos", "aarch64") => "darwin-arm64".to_string(),
        ("windows", "x86_64") => "windows-amd64".to_string(),
        _ => format!("{}-{}", os, arch),
    }
}

fn find_platform_asset<'a>(assets: &'a [serde_json::Value], platform: &str) -> Result<&'a serde_json::Value> {
    // Try to find exact match first
    for asset in assets {
        if let Some(name) = asset["name"].as_str() {
            if name.contains(platform) {
                return Ok(asset);
            }
        }
    }
    
    // If no exact match, try partial matches
    let os = std::env::consts::OS;
    for asset in assets {
        if let Some(name) = asset["name"].as_str() {
            if name.contains(os) {
                return Ok(asset);
            }
        }
    }
    
    // If no platform-specific asset found, try to find a generic binary
    for asset in assets {
        if let Some(name) = asset["name"].as_str() {
            // Check if it's likely a binary (no extension or common binary extensions)
            if !name.contains('.') || name.ends_with(".exe") || name.ends_with(".bin") {
                return Ok(asset);
            }
        }
    }
    
    // If still nothing found, just take the first asset
    if !assets.is_empty() {
        return Ok(&assets[0]);
    }
    
    Err(anyhow!("No assets found in release"))
}

fn is_in_path(directory: &PathBuf) -> Result<bool> {
    let path_env = std::env::var("PATH").unwrap_or_default();
    let paths: Vec<&str> = path_env.split(':').collect();
    
    let dir_str = directory.to_string_lossy();
    
    for path in paths {
        // Handle both exact matches and resolved paths
        if path == dir_str {
            return Ok(true);
        }
        
        // Also check if the path resolves to the same directory
        if let Ok(path_buf) = PathBuf::from(path).canonicalize() {
            if let Ok(dir_canonical) = directory.canonicalize() {
                if path_buf == dir_canonical {
                    return Ok(true);
                }
            }
        }
    }
    
    Ok(false)
}

fn expand_install_path(path: &str) -> Result<PathBuf> {
    if path.starts_with("~/") {
        if let Some(home) = home_dir() {
            let expanded = path.replace("~/", "");
            Ok(home.join(expanded))
        } else {
            Err(anyhow!("Could not determine home directory"))
        }
    } else {
        Ok(PathBuf::from(path))
    }
}

async fn uninstall_tool(tool_name: &str) -> Result<()> {
    let registry = load_tool_registry().await?;
    
    let tool = registry.tools.get(tool_name)
        .ok_or_else(|| anyhow!("Tool '{}' not found in registry", tool_name))?;

    println!("🗑️  Uninstalling {}...", tool.name);

    // Determine the install path
    let install_path = expand_install_path(&tool.install_path)?;
    let binary_path = install_path.join(&tool.binary_name);

    // Check if the binary exists
    if !binary_path.exists() {
        println!("ℹ️  Tool '{}' is not installed at {}", tool.name, binary_path.display());
        return Ok(());
    }

    // Remove the binary
    fs::remove_file(&binary_path)?;
    
    println!("✅ Successfully uninstalled {} from {}", tool.name, binary_path.display());

    Ok(())
}

async fn list_tools() -> Result<()> {
    let registry = load_tool_registry().await?;
    
    println!("📋 Available tools:");
    for (name, tool) in &registry.tools {
        println!("  🔧 {} - {}", name, tool.description);
    }
    
    Ok(())
}
