# git2_auth

An authentication handler to be used with `git2_rs`. An example usage can be seen below. I used the following repos:

- [Cargo](https://github.com/rust-lang/cargo/blob/f72f8a87c8c6ec05a3706ef9987899cd105db622/src/cargo/sources/git/utils.rs#L450-L718)
- [git2_credentials](https://github.com/davidB/git2_credentials)

## TODO

- [ ] User defined ssh handlers
- [ ] User provided username trials

## Example Usage

```rust
// Setup remote callbacks
let mut callback = git2::RemoteCallbacks::new();
let config = git2::Config::open_default().unwrap();
// Setup authentication handler
let mut auth_handler = AuthHandler::default_with_config(config);
callback.credentials(move |url, username, allowed| {
    auth_handler.handle_callback(url, username, allowed)
});
// Create fetch options
let mut fetch_options = git2::FetchOptions::new();
fetch_options
    .remote_callbacks(callback)
    .download_tags(git2::AutotagOption::All)
    .update_fetchhead(true);
// Clone the repo
git2::build::RepoBuilder::new()
    .branch("master")
    .fetch_options(fetch_options)
    .clone("git@github.com:kayagokalp/git2_auth.git", dir.as_ref())
    .unwrap();
```

## License

[MIT](https://choosealicense.com/licenses/mit/)
