use std::{collections::VecDeque, env, path::PathBuf};

type Username = String;
type GitURL = String;

const USERNAME_EMPTY: &str = "";
const USERNAME_GIT: &str = "git";

/// Holds all required information for handling authentication callbacks from `git2`.
pub struct AuthContext<'a> {
    config: &'a git2::Config,
    /// Set of usernames to try in case the username is not specified with the callback.
    usernames: VecDeque<Username>,
    /// Set of methods to try for credential generation using SSH.
    ssh_trial_methods: VecDeque<SSHTrialContext<'a>>,
    /// The url provided by the callback.
    pub callback_url: Option<&'a str>,
    /// The username provieded by the callback.
    pub callback_username: Option<&'a str>,
}

/// Represents supported methods of SSH credential generation.
pub enum SSHTrialContext<'a> {
    /// In this setup, SSH setup stage will try to generate SSH credential using the username.
    Agent,
    Host(HostSSHContext),
    File(FileSSHContext<'a>),
}

/// Holds all required information for handling SSH credential generation from git-url.
pub struct HostSSHContext {
    url: GitURL,
}

impl HostSSHContext {
    pub fn new(url: GitURL) -> Self {
        Self { url }
    }
}

/// Holds all required information for handling SSH credential generation from possible key path.
pub struct FileSSHContext<'a> {
    paths: VecDeque<PathBuf>,
    password: &'a str,
}

impl<'a> FileSSHContext<'a> {
    pub fn new(paths: VecDeque<PathBuf>, password: &'a str) -> Self {
        Self { paths, password }
    }
}

impl<'a> AuthContext<'a> {
    pub fn new(
        config: &'a git2::Config,
        usernames: VecDeque<Username>,
        ssh_trial_methods: VecDeque<SSHTrialContext<'a>>,
        callback_url: Option<&'a str>,
        callback_username: Option<&'a str>,
    ) -> Self {
        Self {
            config,
            usernames,
            ssh_trial_methods,
            callback_url,
            callback_username,
        }
    }

    /// Creates a new `AuthContext` with provided `git2::Config` and default values for other
    /// context used during handling authentication callbacks.
    pub fn default_with_config(config: &'a git2::Config) -> Self {
        // If username is not specified, tries the following sources:
        //  1. Empty string ""
        //  2. "git"
        //  3. Fetch username from env
        let mut usernames = VecDeque::with_capacity(3);
        usernames.push_back(USERNAME_EMPTY.to_string());
        usernames.push_back(USERNAME_GIT.to_string());
        if let Ok(env_username) = env::var("USER") {
            usernames.push_back(env_username);
        }
        let ssh_trial_method = VecDeque::default();
        let callback_url = None;
        let callback_username = None;
        Self::new(
            config,
            usernames,
            ssh_trial_method,
            callback_url,
            callback_username,
        )
    }

    /// Removes and returns the next username to from auth context.
    pub fn get_last_username(&mut self) -> Option<Username> {
        let usernames = &mut self.usernames;
        usernames.pop_front()
    }

    pub fn get_last_ssh_trial_method(&mut self) -> Option<SSHTrialContext<'a>> {
        let methods = &mut self.ssh_trial_methods;
        methods.pop_front()
    }

    pub(crate) fn handle_username_callback(&mut self) -> Result<git2::Cred, git2::Error> {
        let username = self.get_last_username().ok_or_else(|| {
            git2::Error::from_str("tried all possible usernames for the callback")
        })?;
        git2::Cred::username(&username)
    }

    pub(crate) fn handle_ssh_callback(&mut self) -> Result<git2::Cred, git2::Error> {
        let ssh_trial_method = self
            .get_last_ssh_trial_method()
            .ok_or_else(|| git2::Error::from_str("no ssh handler present for authentication"))?;
        ssh_trial_method.handle_callback(self.callback_url, self.callback_username)
    }
}

impl<'a> SSHTrialContext<'a> {
    pub(crate) fn handle_callback(
        &self,
        _callback_url: Option<&str>,
        callback_username: Option<&str>,
    ) -> Result<git2::Cred, git2::Error> {
        match self {
            SSHTrialContext::Agent => {
                // SSH authentication is with agent is going to be attempted, this means callback
                // must be providing a username.
                let username = callback_username.ok_or_else(|| {
                    git2::Error::from_str("username must be provided with SSH_KEY callback")
                })?;
                git2::Cred::ssh_key_from_agent(username)
            }
            SSHTrialContext::Host(_) => todo!(),
            SSHTrialContext::File(_) => todo!(),
        }
    }
}
