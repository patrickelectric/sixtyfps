use std::fs::File;
use std::io::Write;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(name = "path to .60 file", parse(from_os_str))]
    path: std::path::PathBuf,

    #[structopt(name = "output directory", parse(from_os_str))]
    output_directory: std::path::PathBuf,
}

fn write_file_if_changed(file: &std::path::Path, new_contents: &[u8]) -> std::io::Result<()> {
    if let Ok(old_contents) = std::fs::read_to_string(&file) {
        if old_contents.as_bytes() == new_contents {
            return Ok(());
        }
    }
    let mut f = File::create(file)?;
    f.write_all(new_contents)
}

fn main() -> std::io::Result<()> {
    let args = Cli::from_args();

    {
        let mut web_api_path = std::env::current_exe().unwrap();
        web_api_path.pop(); // pop of executable name
        web_api_path.push("..");
        web_api_path.push("..");
        web_api_path.push("api");
        web_api_path.push("sixtyfps-rs");

        let cargo_template: String = include_str!("Cargo_toml_template.txt").into();
        let cargo_template =
            cargo_template.replace("${WEB_API_PATH}", web_api_path.as_path().to_str().unwrap());

        let mut cargo_toml_path = args.output_directory.clone();
        cargo_toml_path.push("Cargo.toml");

        write_file_if_changed(cargo_toml_path.as_path(), cargo_template.as_bytes())?;
    }

    {
        let source = std::fs::read_to_string(&args.path)?;

        let main_rs_template: String = include_str!("main_rs_template.txt").into();
        let main_rs_template = main_rs_template.replace("${SOURCE_CODE}", &source);

        let mut main_rs_path = args.output_directory.clone();
        main_rs_path.push("main.rs");

        write_file_if_changed(main_rs_path.as_path(), main_rs_template.as_bytes())?;
    }

    {
        let mut wasm_pack = Command::new("wasm-pack")
            .args(&["build"])
            .current_dir(args.output_directory)
            .spawn()
            .expect("Spawning wasm-pack failed");
        wasm_pack.wait().expect("Running wasm-pack failed");
    }

    println!("Build complete.");
    Ok(())
}
