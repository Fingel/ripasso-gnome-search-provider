use arboard::Clipboard;
use ripasso::pass::PasswordStore;
use search_provider::{ResultID, ResultMeta, SearchProvider, SearchProviderImpl};
use std::{collections::HashMap, error::Error, path::PathBuf, thread, time};
use totp_rs::TOTP;
use zbus::{blocking::Connection, proxy, zvariant::Value};
use zeroize::Zeroize;

#[proxy(
    default_service = "org.freedesktop.Notifications",
    default_path = "/org/freedesktop/Notifications"
)]
trait Notifications {
    fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: &[&str],
        hints: HashMap<&str, &Value<'_>>,
        expire_timeout: i32,
    ) -> zbus::Result<u32>;
}

fn copy_to_clipbard(content: &String) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(content).unwrap();
    thread::spawn(|| {
        thread::sleep(time::Duration::from_secs(40));
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.set_text(&String::new()).unwrap();
    });
}

fn send_notification(summary: String, body: String) {
    thread::spawn(move || {
        let connection = Connection::session().unwrap();
        let proxy = NotificationsProxyBlocking::new(&connection).unwrap();
        let _ = proxy.notify(
            "Pass",
            0,
            "dialog-password",
            &summary,
            &body,
            &[],
            HashMap::from([("transient", &Value::Bool(true))]),
            4000,
        );
    })
    .join()
    .unwrap();
}

/// decrypts and returns a TOTP code if the entry contains a otpauth:// url
/// # Errors
/// Returns an `Err` if the code generation fails
/// TODO: Remove this when/if https://github.com/cortex/ripasso/pull/358/
/// is merged.
pub fn mfa(secret: &String) -> Result<String, Box<dyn Error>> {
    if let Some(start_pos) = secret.find("otpauth://") {
        let end_pos = {
            let mut end_pos = secret.len();
            for (pos, c) in secret.chars().skip(start_pos).enumerate() {
                if c.is_whitespace() {
                    end_pos = pos + start_pos;
                    break;
                }
            }
            end_pos
        };
        // Use unchecked for sites like Discord, Github that still use 80
        // bit secrets. https://github.com/constantoine/totp-rs/issues/46
        let totp = TOTP::from_url_unchecked(&secret[start_pos..end_pos])?;
        Ok(totp.generate_current()?)
    } else {
        Err("No OTP URL found".into())
    }
}

struct Application {
    password_store: PasswordStore,
}

impl SearchProviderImpl for Application {
    fn activate_result(&self, identifier: ResultID, terms: &[String], _timestamp: u32) {
        let entries = self.password_store.all_passwords().unwrap_or_default();
        if let Some(entry) = entries
            .iter()
            .find(|entry| entry.name == identifier.to_owned())
        {
            if terms[0] == "otp" {
                match entry.secret(&self.password_store) {
                    Ok(mut secret) => {
                        match mfa(&secret) {
                            Ok(mut otp) => {
                                copy_to_clipbard(&otp);
                                otp.zeroize();
                                send_notification(
                                    identifier,
                                    "OTP copied to clipboard".to_string(),
                                );
                            }
                            Err(err) => {
                                send_notification("OTP Error".to_string(), err.to_string());
                            }
                        };
                        secret.zeroize();
                    }
                    Err(err) => {
                        send_notification("Could not read entry".to_string(), err.to_string())
                    }
                };
            } else {
                match entry.password(&self.password_store) {
                    Ok(mut password) => {
                        copy_to_clipbard(&password);
                        password.zeroize();
                        send_notification(identifier, "Password copied to clipboard".to_string())
                    }
                    Err(err) => send_notification("Password Error".to_string(), err.to_string()),
                };
            }
        } else {
            send_notification("Error".to_string(), "Could Not Find Password".to_string())
        }
    }

    fn initial_result_set(&self, terms: &[String]) -> Vec<ResultID> {
        let search_terms = if terms[0] == "otp" {
            &terms[1..]
        } else {
            terms
        };

        self.password_store
            .all_passwords()
            .unwrap_or_default()
            .iter()
            .filter(|entry| {
                search_terms
                    .iter()
                    .any(|term| entry.name.to_lowercase().contains(&term.to_lowercase()))
            })
            .map(|entry| entry.name.to_owned())
            .collect()
    }

    fn result_metas(&self, identifiers: &[ResultID]) -> Vec<ResultMeta> {
        identifiers
            .iter()
            .map(|id| ResultMeta::builder(id.to_owned(), id).build())
            .collect()
    }
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let home = std::env::var("HOME").expect("Could not determine $HOME");
    let home_path = PathBuf::from(home);
    let default_path = match std::env::var("PASSWORD_STORE_DIR") {
        Ok(val) => PathBuf::from(val),
        Err(_) => [home_path.to_str().unwrap(), ".password-store"]
            .iter()
            .collect(),
    };
    let password_store = PasswordStore::new(
        "default",
        &Some(default_path),
        &None,
        &Some(home_path),
        &None,
        &ripasso::crypto::CryptoImpl::GpgMe,
        &None,
    )
    .unwrap();
    let app = Application { password_store };
    SearchProvider::new(
        app,
        "io.m51.Pass.SearchProvider",
        "/io/m51/Pass/SearchProvider",
    )
    .await?;
    Ok(())
}
