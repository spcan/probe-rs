use std::env;
use xshell::cmd;

type DynError = Box<dyn std::error::Error>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let task = env::args().nth(1);
    match task.as_ref().map(|it| it.as_str()) {
        Some("release") => release(
            &env::args()
                .nth(2)
                .ok_or("Please give me the version of the next release (e.g. 0.2.0).")?,
        )?,
        Some("fetch-prs") => fetch_prs()?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:
fetch-prs
    Help: Fetches all the PRs since the current release.

release <next_release>
    Arguments:
        - <next_release>: The version number of the next to be released version on crates.io (e.g. 0.2.0)
    Help: Performs the following steps to trigger a new release:
        1. Bump all probe-rs dependency numbers.
        2. Create a commit.
        3. Create a PR with a label.
"
    )
}

fn fetch_prs() -> Result<(), DynError> {
    // Make sure we are on the master branch and we have the latest state pulled from our source of truth, GH.
    cmd!("gh pr list --label 'needs-changelog' --state 'closed' --web --limit 300").run()?;

    Ok(())
}

fn release(version: &str) -> Result<(), DynError> {
    // Make sure we are on the master branch and we have the latest state pulled from our source of truth, GH.
    cmd!("git checkout master").run()?;
    cmd!("git pull").run()?;

    // Bump the crate versions.
    cmd!("cargo workspaces version -y --no-git-commit custom {version}").run()?;

    // Checkout a release branch
    cmd!("git checkout -b v{version}").run()?;

    // Create the release commit.
    let message = format!("Prepare for the v{} release.", version);
    cmd!("git commit -a -m {message}").run()?;
    cmd!("git push -u origin v{version}").run()?;

    // Create the PR with a proper label, which then gets picked up by the CI.
    let message = format!(
        "Bump probe-rs versions in preparation for the v{} release.",
        version
    );
    cmd!("gh pr create --label 'release' --title {version} --repo 'probe-rs/probe-rs' --body {message}").run()?;

    Ok(())
}
