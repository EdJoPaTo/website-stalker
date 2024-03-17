use url::Url;

use crate::logger::gha_output;

#[derive(serde::Serialize)]
pub struct Summary {
    pub change: bool,
    pub siteamount: usize,
    pub commit: Option<String>,
    pub singlehost: Option<String>,
    pub hosts: Vec<String>,
    pub sites: Vec<Url>,
}

impl Summary {
    pub fn new(commit: Option<String>, changed_urls: Vec<Url>) -> Self {
        let mut sites = changed_urls;
        sites.sort_unstable();
        sites.dedup();

        let mut hosts = sites
            .iter()
            .filter_map(Url::host_str)
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        hosts.dedup();

        let singlehost = if let [single] = hosts.as_slice() {
            Some(single.clone())
        } else {
            None
        };

        Self {
            change: !sites.is_empty(),
            siteamount: sites.len(),
            commit,
            singlehost,
            hosts,
            sites,
        }
    }

    pub fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(&self)
            .expect("Should be able to turn summary into valid pretty JSON")
    }

    pub fn to_gha_output(&self) {
        gha_output("change", &self.change.to_string());
        gha_output("siteamount", &self.siteamount.to_string());
        gha_output_option("commit", self.commit.as_deref());
        gha_output_option("singlehost", self.singlehost.as_deref());

        gha_output("hosts", &to_json(&self.hosts));
        gha_output("sites", &to_json(&self.sites));

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
