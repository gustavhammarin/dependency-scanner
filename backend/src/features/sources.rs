
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum PackageSource {
    Github,
    Nuget,
    Npm,
}

impl Default for PackageSource {
    fn default() -> Self {
        PackageSource::Github
    }
}

impl From<String> for PackageSource {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "github" => PackageSource::Github,
            "nuget" => PackageSource::Nuget,
            "npm" => PackageSource::Npm,
            _ => panic!("Unknown package source: {}", s),
        }
    }
}
