use url::Url;

use crate::logger::gha_output;

#[derive(serde::Serialize)]
pub struct Summary {
    pub change: bool,
    pub changed_amount: usize,

    pub any_failed: bool,
    pub failed_amount: usize,

    pub commit: Option<String>,

    pub changed_hosts: Vec<String>,
    pub changed_sites: Vec<Url>,
    pub failed_sites: Vec<Url>,
}

impl Summary {
    pub fn new(commit: Option<String>, mut changed: Vec<Url>, mut failed: Vec<Url>) -> Self {
        changed.sort_unstable();
        changed.dedup();

        failed.sort_unstable();
        failed.dedup();

        let mut changed_hosts = changed
            .iter()
            .filter_map(Url::host_str)
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        changed_hosts.dedup();

        Self {
            change: !changed.is_empty(),
            changed_amount: changed.len(),

            any_failed: !failed.is_empty(),
            failed_amount: failed.len(),

            commit,

            changed_hosts,
            changed_sites: changed,
            failed_sites: failed,
        }
    }

    pub fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(&self)
            .expect("Should be able to turn summary into valid pretty JSON")
    }

    pub fn to_gha_output(&self) {
        gha_output("change", &self.change.to_string());
        gha_output("changed-amount", &self.changed_amount.to_string());

        gha_output("any-failed", &self.any_failed.to_string());
        gha_output("failed-amount", &self.failed_amount.to_string());

        gha_output_option("commit", self.commit.as_deref());

        gha_output("changed-hosts", &to_json(&self.changed_hosts));
        gha_output("changed-sites", &to_json(&self.changed_sites));
        gha_output("failed-sites", &to_json(&self.failed_sites));

        gha_output("json", &to_json(self));
    }
}

fn gha_output_option(key: &str, text: Option<&str>) {
    if let Some(text) = text {
        gha_output(key, text);
    }
}

fn to_json<T>(value: &T) -> String
where
    T: ?Sized + serde::Serialize,
{
    serde_json::to_string(value).expect("Should be able to turn summary into valid JSON")
}
