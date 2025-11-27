use clap::Parser;
use git2::{AutotagOption, Cred, FetchOptions, RemoteCallbacks, Repository};
use std::process::exit;

use crate::cli::common;

#[derive(Parser, Debug, Default)]
pub struct Args {
    // name of the tag
    // #[arg()]
    // tag: String,
}

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    println!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;
    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        println!("Merge conflicts detected...");
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(())
}

pub fn execute(_args: Args) {
    let repo = common::get_araki_git_repo().unwrap_or_else(|err| {
        eprintln!("Couldn't recognize the araki repo: {err}");
        exit(1);
    });
    let mut remote = repo.find_remote("origin").unwrap();

    let mut callbacks = RemoteCallbacks::new();
    // TODO: allow user to configure their ssh key
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key_from_agent(username_from_url.unwrap())
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);
    fetch_opts.download_tags(AutotagOption::All);

    // Pull changes
    remote
        .fetch(&["main"], Some(&mut fetch_opts), None)
        .expect("Unable to pull from remote");

    let fetch_head = repo.find_reference("FETCH_HEAD");
    let fetch_commit = repo
        .reference_to_annotated_commit(&fetch_head.unwrap())
        .unwrap();

    // ref: https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
    // Determine type of merge
    let analysis = repo.merge_analysis(&[&fetch_commit]).unwrap();
    // 2. Do the appropriate merge
    if analysis.0.is_fast_forward() {
        println!("Doing a fast forward");
        // do a fast forward
        let refname = format!("refs/heads/{}", "main");
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                fast_forward(&repo, &mut r, &fetch_commit).expect("Unable to fast forward");
            }
            Err(_) => {
                // The branch doesn't exist so just set the reference to the
                // commit directly. Usually this is because you are pulling
                // into an empty repository.
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", "main", fetch_commit.id()),
                )
                .unwrap();
                repo.set_head(&refname).unwrap();
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))
                .expect("Unable to checkout head");
            }
        };
    } else if analysis.0.is_normal() {
        // do a normal merge
        let head_commit = repo
            .reference_to_annotated_commit(&repo.head().unwrap())
            .unwrap();
        normal_merge(&repo, &head_commit, &fetch_commit).expect("Unable to merge");
    }
}
