use std::{collections::VecDeque, env};

type Username = String;

const USERNAME_EMPTY: &str = "";
const USERNAME_GIT: &str = "git";

/// Handler holds all required information for handling authentication callbacks from `git2`.
pub struct AuthHandler {
    config: git2::Config,
    /// Set of usernames to try in case the username is not specified with the callback.
    usernames: VecDeque<Username>,
    /// Set of methods to try for credential generation using SSH.
    ssh_trial_methods: VecDeque<SSHTrialMethod>,
    /// Shows if the `AuthHandler` tried to use `USER-PASS-PLAINTEXT` before, to prevent looping.
    tried_plain_user_pass: bool,
    /// The username provieded by the callback.
    pub callback_username: Option<Username>,
}

/// Represents supported methods of SSH credential generation.
///
/// TODO: Convert this into a trait so that downstream can add new methods
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SSHTrialMethod {
    /// In this setup, SSH setup stage will try to generate SSH credential using the username.
    Agent,
}

impl AuthHandler {
    /// Creates a new `AuthHandler` from all fields of the struct. If there are no specific reasons
    /// not to, `default_with_config` should be prefered.
    pub fn new(
        config: git2::Config,
        usernames: VecDeque<Username>,
        ssh_trial_methods: VecDeque<SSHTrialMethod>,
        tried_plain_user_pass: bool,
        callback_username: Option<String>,
    ) -> Self {
        Self {
            config,
            usernames,
            ssh_trial_methods,
            tried_plain_user_pass,
            callback_username,
        }
    }

    /// Creates a new `AuthContext` with provided `git2::Config` and default values for other
    /// context used during handling authentication callbacks.
    pub fn default_with_config(config: git2::Config) -> Self {
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
        // By default try to do SSH authentication from:
        //  1. Agent
        let mut ssh_trial_method = VecDeque::default();
        ssh_trial_method.push_back(SSHTrialMethod::Agent);
        let callback_username = None;
        let tried_plain_user_pass = false;
        Self::new(
            config,
            usernames,
            ssh_trial_method,
            tried_plain_user_pass,
            callback_username,
        )
    }

    /// Handle a git2 remote credential callback dependening on the current state of the handler.
    /// This dispatches the callback to the correct handler. For example:
    ///
    /// 1. `git2::CredentialType::USERNAME` calls are dispathced to `handle_username_callback()`
    /// 2. `git2::CredentialType::SSH_KEY` calls are dispatched to `handle_ssh_callback()`
    pub fn handle_callback(
        &mut self,
        url: &str,
        username: Option<&str>,
        allowed: git2::CredentialType,
    ) -> Result<git2::Cred, git2::Error> {
        self.callback_username = username.map(|st| st.to_string());

        // The username is missing and we will try possbilities from context.
        if allowed.contains(git2::CredentialType::USERNAME) {
            return self.handle_username_callback();
        }
        if allowed.contains(git2::CredentialType::SSH_KEY) && !self.ssh_trial_methods.is_empty() {
            return self.handle_ssh_callback();
        }
        if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            return git2::Cred::credential_helper(&self.config, url, username);
        }
        if allowed.contains(git2::CredentialType::DEFAULT) && !self.tried_plain_user_pass {
            self.tried_plain_user_pass = true;
            return git2::Cred::default();
        }

        Err(git2::Error::from_str(
            "tried all possible credential types for authentication",
        ))
    }

    /// Removes and returns the next username to from this `AuthHandler`.
    pub fn get_next_username(&mut self) -> Option<Username> {
        let usernames = &mut self.usernames;
        usernames.pop_front()
    }

    /// Removes and returns the next ssh trial method to from this `AuthHandler`.
    pub fn get_next_ssh_trial_method(&mut self) -> Option<SSHTrialMethod> {
        let methods = &mut self.ssh_trial_methods;
        methods.pop_front()
    }

    /// Handles a `git2::CredentialType::USERNAME` callback and tries to generate a credential from
    /// all possible username options the handler currently have.
    ///
    /// If this `AuthHandler` is created with `default_with_config` the options are:
    ///
    /// 1. Empty string ("")
    /// 2. "git"
    /// 3. $USER from env
    ///
    /// This handler is used if the callback does not provide a username. That happens when the
    /// username cannot be infered from the url.
    pub(crate) fn handle_username_callback(&mut self) -> Result<git2::Cred, git2::Error> {
        let username = self.get_next_username().ok_or_else(|| {
            git2::Error::from_str("tried all possible usernames for the callback")
        })?;
        git2::Cred::username(&username)
    }

    /// Handles a `git2::CredentialType::SSH_KEY` callback and tries to generate a credential from
    /// all possible SSH trial methods the handler currently have.
    ///
    /// If this `AuthHandler` is created iwth `default_with_config` the options are:
    ///
    /// 1. Agent
    ///
    /// This handler dispatches the callback to the current method's handler.
    pub(crate) fn handle_ssh_callback(&mut self) -> Result<git2::Cred, git2::Error> {
        let ssh_trial_method = self
            .get_next_ssh_trial_method()
            .ok_or_else(|| git2::Error::from_str("no ssh handler present for authentication"))?;
        ssh_trial_method.handle_callback(self.callback_username.as_ref())
    }
}

impl SSHTrialMethod {
    /// Handles the dispatched `git2::CredentialType::SSH_KEY` depending on the current method the
    /// handler is trying.
    pub(crate) fn handle_callback(
        &self,
        callback_username: Option<&Username>,
    ) -> Result<git2::Cred, git2::Error> {
        match self {
            SSHTrialMethod::Agent => {
                // SSH authentication is with agent is going to be attempted, this means callback
                // must be providing a username.
                let username = callback_username.ok_or_else(|| {
                    git2::Error::from_str("username must be provided with SSH_KEY callback")
                })?;
                git2::Cred::ssh_key_from_agent(username)
            }
        }
    }
}
