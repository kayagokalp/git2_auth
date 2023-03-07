pub mod auth_handler;
mod utils;

#[cfg(test)]
mod tests {
    use crate::{auth_handler::AuthHandler, utils::tests_utils::with_tmp_dir};

    #[test]
    fn fetch_with_git_url() {
        with_tmp_dir(|dir| {
            // setup callbacks
            let mut callback = git2::RemoteCallbacks::new();
            let config = git2::Config::open_default().unwrap();
            let mut auth_handler = AuthHandler::default_with_config(config);
            callback.credentials(move |url, username, allowed| {
                auth_handler.handle_callback(url, username, allowed)
            });

            let mut fetch_options = git2::FetchOptions::new();
            fetch_options
                .remote_callbacks(callback)
                .download_tags(git2::AutotagOption::All)
                .update_fetchhead(true);

            git2::build::RepoBuilder::new()
                .branch("master")
                .fetch_options(fetch_options)
                .clone("git@github.com:kayagokalp/handtrack-rs.git", dir.as_ref())
                .unwrap();
        });
    }
}
