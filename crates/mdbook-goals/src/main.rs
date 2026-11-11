use clap::Parser;
use goal_preprocessor::GoalPreprocessor;
use mdbook_preprocessor::Preprocessor;
use semver::{Version, VersionReq};
use std::io;

mod goal_preprocessor;

#[derive(clap::Parser, Debug)]
#[structopt(about = "Project goal preprocessor")]
struct Opt {
    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(clap::Subcommand, Debug)]
#[allow(dead_code)]
enum Command {
    /// Command used by mdbook to check if the preprocessor supports a renderer
    Supports { renderer: String },
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    let Some(cmd) = &opt.cmd else {
        return handle_preprocessing(&GoalPreprocessor);
    };

    match cmd {
        Command::Supports { renderer } => {
            handle_supports(&GoalPreprocessor, renderer)?;
        }
    }

    Ok(())
}

// from https://github.com/rust-lang/mdBook/blob/master/examples/nop-preprocessor.rs
fn handle_supports(pre: &dyn Preprocessor, renderer: &str) -> anyhow::Result<()> {
    let supported = pre.supports_renderer(renderer)?;

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        Ok(())
    } else {
        anyhow::bail!("renderer `{renderer}` unsupported")
    }
}

// from https://github.com/rust-lang/mdBook/blob/master/examples/nop-preprocessor.rs
fn handle_preprocessing(pre: &dyn Preprocessor) -> anyhow::Result<()> {
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook_preprocessor::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook_preprocessor::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
