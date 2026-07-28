//! One-shot tool that registers an ACME account and prints its credentials
//! JSON to stdout, to be deployed as the `ACME_ACCOUNT_CREDENTIALS` secret.
//! Reads `ACME_DIRECTORY_URL` (required), `ACME_CONTACT` and
//! `ACME_ROOT_CA_PATH`

use std::path::PathBuf;

fn env_opt(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}

#[tokio::main]
async fn main() {
    let Some(directory_url) = env_opt("ACME_DIRECTORY_URL") else {
        eprintln!("ACME_DIRECTORY_URL must be set to the CA directory to register with");
        std::process::exit(1);
    };
    let contact = env_opt("ACME_CONTACT");
    let root_ca_path = env_opt("ACME_ROOT_CA_PATH").map(PathBuf::from);

    match eks::create_acme_account(directory_url, contact.as_deref(), root_ca_path.as_deref()).await
    {
        Ok(credentials_json) => println!("{credentials_json}"),
        Err(err) => {
            eprintln!("could not create the ACME account: {err}");
            std::process::exit(1);
        }
    }
}
