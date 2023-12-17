// https://github.com/rust-lang/docs.rs/blob/7fdd5d839cb68d703c2732d784aa12692d58ab54/build.rs

cfg_if::cfg_if! {
  if #[cfg(feature = "accessory")] {
    const TEMPLATES_DIRECTORY: &str = "docs.rs/templates";
    const STATIC_DIRECTORY: &str = "docs.rs/static";

    use anyhow::{Context as _, Result};
    use std::{env, path::Path};
    use crate::{
      git_version::write_git_version,
      style::{compile_sass, compile_syntax},
      templates::generate_code,
    };
  } else {
    use anyhow::Result;
  }
}

fn main() -> Result<()> {
    #[cfg(feature = "accessory")]
    {
        let out_dir = env::var("OUT_DIR").context("missing OUT_DIR")?;
        let out_dir = Path::new(&out_dir);

        write_git_version(out_dir)?;

        compile_sass(out_dir)?;

        compile_syntax(out_dir).context("could not compile syntax files")?;

        generate_code()?;

        println!("cargo:rerun-if-changed=build.rs");
    }

    Ok(())
}

#[cfg(feature = "accessory")]
mod templates {
    use std::path::Path;

    use path_slash::PathExt;
    use walkdir::WalkDir;

    use crate::{STATIC_DIRECTORY, TEMPLATES_DIRECTORY};

    use anyhow::Result;

    fn find_templates_in_filesystem(base: &str) -> Vec<(String, String)> {
        let root = std::fs::canonicalize(base).expect("failed to canonicalize templates directory");

        let mut files = Vec::new();
        for entry in WalkDir::new(&root) {
            let entry = entry.expect("failed to read template directory entry");
            let path = entry.path().strip_prefix(&root).unwrap();

            if !entry.metadata().unwrap().is_file() {
                continue;
            }

            // Strip the root directory from the path and use it as the template name.
            let name = path.to_slash().expect("failed to normalize template path");

            let joined_path = Path::new("../../").join(base).join(path);
            let path = joined_path
                .to_slash()
                .expect("failed to normalize template path");

            files.push((path.to_string(), name.to_string()));
        }

        files
    }

    pub fn generate_code() -> Result<()> {
        let mut code = String::new();
        code.push_str(&generate_include_str_templates_code()?);
        code.push_str(&generate_include_str_static_code()?);

        std::fs::write("src/docs_rs/generated_code.rs", code)?;

        Ok(())
    }

    fn generate_include_str_templates_code() -> Result<String> {
        let files = find_templates_in_filesystem(TEMPLATES_DIRECTORY);
        let mut code = String::new();
        code.push_str("pub fn raw_templates() -> Vec<(&'static str, &'static str)> {\n");
        code.push_str("  vec![\n");
        for (path, name) in files {
            code.push_str(&format!(
                "    (\"{}\", include_str!(\"{}\")),\n",
                name, path
            ));
        }
        code.push_str("  ]\n");
        code.push_str("}\n");
        Ok(code)
    }

    fn generate_include_str_static_code() -> Result<String> {
        let files = find_templates_in_filesystem(STATIC_DIRECTORY);
        let mut code = String::new();
        code.push_str("pub fn raw_static() -> Vec<(&'static str, &'static [u8])> {\n");
        code.push_str("  vec![\n");
        for (path, name) in files {
            code.push_str(&format!(
                "    (\"{}\", include_bytes!(\"{}\")),\n",
                name, path
            ));
        }
        code.push_str("  ]\n");
        code.push_str("}\n");
        Ok(code)
    }
}

#[cfg(feature = "accessory")]
mod git_version {
    use anyhow::Result;
    use std::{env, path::Path};

    use crate::tracked;

    pub fn write_git_version(out_dir: &Path) -> Result<()> {
        let maybe_hash = get_git_hash()?;
        let git_hash = maybe_hash.as_deref().unwrap_or("???????");

        let build_date = time::OffsetDateTime::now_utc().date();

        std::fs::write(
            out_dir.join("git_version"),
            format!("({git_hash} {build_date})"),
        )?;

        Ok(())
    }

    fn get_git_hash() -> Result<Option<String>> {
        match gix::open_opts(env::current_dir()?, gix::open::Options::isolated()) {
            Ok(repo) => {
                let head_id = repo.head()?.id();

                // TODO: are these right?
                tracked::track(".git/HEAD")?;
                tracked::track(".git/index")?;

                Ok(head_id.map(|h| format!("{}", h.shorten_or_id())))
            }
            Err(err) => {
                eprintln!("failed to get git repo: {err}");
                Ok(None)
            }
        }
    }
}

#[cfg(feature = "accessory")]
mod tracked {
    use once_cell::sync::Lazy;
    use path_slash::PathBufExt;
    use std::{
        collections::HashSet,
        io::{Error, ErrorKind, Result},
        path::{Path, PathBuf},
        sync::Mutex,
    };

    static SEEN: Lazy<Mutex<HashSet<PathBuf>>> = Lazy::new(|| Mutex::new(HashSet::new()));

    pub(crate) fn track(path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if path.exists() {
            let mut seen = SEEN.lock().unwrap();
            // TODO: Needs something like `HashSet::insert_owned` to check before cloning
            // https://github.com/rust-lang/rust/issues/60896
            if !seen.contains(path) {
                seen.insert(path.to_owned());
                let path = path.to_path_buf();
                let path = path.to_slash().ok_or_else(|| {
                    Error::new(
                        ErrorKind::Other,
                        format!("{} is a non-utf-8 path", path.display()),
                    )
                })?;
                println!("cargo:rerun-if-changed={path}");
            }
        } else if let Some(parent) = path.parent() {
            // if the file doesn't exist, we need to notice if it begins existing
            track(parent)?;
        }
        Ok(())
    }

    pub(crate) fn track_recursive(path: impl AsRef<Path>) -> Result<()> {
        for entry in walkdir::WalkDir::new(path) {
            track(entry?.path())?;
        }
        Ok(())
    }

    pub(crate) fn read(path: impl AsRef<Path>) -> Result<Vec<u8>> {
        let path = path.as_ref();
        track(path)?;
        std::fs::read(path)
    }

    pub(crate) fn read_to_string(path: impl AsRef<Path>) -> Result<String> {
        let path = path.as_ref();
        track(path)?;
        std::fs::read_to_string(path)
    }

    #[derive(Debug)]
    pub(crate) struct Fs;

    impl grass::Fs for Fs {
        fn is_dir(&self, path: &Path) -> bool {
            track(path).unwrap();
            path.is_dir()
        }
        fn is_file(&self, path: &Path) -> bool {
            track(path).unwrap();
            path.is_file()
        }
        fn read(&self, path: &Path) -> Result<Vec<u8>> {
            read(path)
        }
    }
}

#[cfg(feature = "accessory")]
mod style {
    use anyhow::{Context as _, Error, Result};
    use std::path::Path;

    use crate::tracked;

    fn compile_sass_file(src: &Path, dest: &Path) -> Result<()> {
        let css = grass::from_path(
            src.to_str()
                .context("source file path must be a utf-8 string")?,
            &grass::Options::default()
                .fs(&tracked::Fs)
                .style(grass::OutputStyle::Compressed),
        )
        .map_err(|e| Error::msg(e.to_string()))?;

        std::fs::write(dest, css)?;

        Ok(())
    }

    pub fn compile_sass(out_dir: &Path) -> Result<()> {
        const STYLE_DIR: &str = "docs.rs/templates/style";

        for entry in walkdir::WalkDir::new(STYLE_DIR) {
            let entry = entry?;
            if entry.metadata()?.is_dir() {
                tracked::track(entry.path())?;
            } else {
                let file_name = entry
                    .file_name()
                    .to_str()
                    .context("file name must be a utf-8 string")?;
                if !file_name.starts_with('_') {
                    let dest = out_dir
                        .join(entry.path().strip_prefix(STYLE_DIR)?)
                        .with_extension("css");
                    compile_sass_file(entry.path(), &dest).with_context(|| {
                        format!("compiling {} to {}", entry.path().display(), dest.display())
                    })?;
                }
            }
        }

        // Compile vendored.css
        let pure = tracked::read_to_string("docs.rs/vendor/pure-css/css/pure-min.css")?;
        let grids =
            tracked::read_to_string("docs.rs/vendor/pure-css/css/grids-responsive-min.css")?;
        let vendored = pure + &grids;
        std::fs::write(out_dir.join("vendored").with_extension("css"), vendored)?;

        Ok(())
    }

    pub fn compile_syntax(out_dir: &Path) -> Result<()> {
        use syntect::{
            dumps::dump_to_uncompressed_file,
            parsing::{SyntaxDefinition, SyntaxSetBuilder},
        };

        fn tracked_add_from_folder(
            builder: &mut SyntaxSetBuilder,
            path: impl AsRef<Path>,
        ) -> Result<()> {
            // There's no easy way to know exactly which files matter, so just track everything in the
            // folder
            tracked::track_recursive(&path)?;
            builder.add_from_folder(path, true)?;
            Ok(())
        }

        let mut builder = SyntaxSetBuilder::new();
        builder.add_plain_text_syntax();

        tracked_add_from_folder(&mut builder, "docs.rs/assets/syntaxes/Packages/")?;

        // The TOML syntax already includes `Cargo.lock` in its alternative file extensions, but we
        // also want to support `Cargo.toml.orig` files.
        let mut toml = SyntaxDefinition::load_from_str(
            &tracked::read_to_string("docs.rs/assets/syntaxes/Extras/TOML/TOML.sublime-syntax")?,
            true,
            Some("TOML"),
        )?;
        toml.file_extensions.push("Cargo.toml.orig".into());
        builder.add(toml);

        tracked_add_from_folder(
            &mut builder,
            "docs.rs/assets/syntaxes/Extras/JavaScript (Babel).sublime-syntax",
        )?;

        dump_to_uncompressed_file(&builder.build(), out_dir.join("syntect.packdump"))?;

        Ok(())
    }
}
