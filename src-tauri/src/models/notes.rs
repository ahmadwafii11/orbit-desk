use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub pinned: bool,
    pub created_at: i64, // Milidetik sejak epoch Unix.
    pub updated_at: i64, // Milidetik sejak epoch Unix yang hanya dapat berubah saat isi diedit.
}