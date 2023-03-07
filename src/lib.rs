mod auth_handler;
use auth_handler::AuthContext;

pub fn handle_auth_callback<'a>(
    auth_context: &'a mut AuthContext<'a>,
    url: &'a str,
    username: Option<&'a str>,
    allowed: git2::CredentialType,
) -> Result<git2::Cred, git2::Error> {
    // Update context with the provided url and username.
    auth_context.callback_username = username;
    auth_context.callback_url = Some(url);

    // The username is missing and we need to try from context.
    if allowed.contains(git2::CredentialType::USERNAME) {
        return auth_context.handle_username_callback();
    } else if allowed.contains(git2::CredentialType::SSH_KEY) {
        return auth_context.handle_ssh_callback();
    }
    todo!()
}
