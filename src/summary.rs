use url::Url;

#[derive(serde::Serialize)]
pub struct Summary {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,

    pub siteamount: usize,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub singlehost: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hosts: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
            commit,
            siteamount: sites.len(),
            singlehost,
            hosts,
            sites,
        }
    }
}
