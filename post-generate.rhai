// Copy template files to root, stripping .template extension
file::rename(".generate/Cargo.toml.template", "Cargo.toml");
file::rename(".generate/README.md.template", "README.md");
file::rename(".generate/release.yaml.template", ".github/workflows/release.yaml");

// Clean up .generate directory
file::delete(".generate");
