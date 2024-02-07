$version = $args[0].replace("v", "")

$files = @(
    "./chip8/Cargo.toml"
    "./chip8-rs/Cargo.toml"
)
for ($file in $files) {
    ((Get-Content -path $file -Raw) -replace '0.0.0', $version) | Set-Content -Path $file
}