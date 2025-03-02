use std::fmt::{self, Display, Formatter};

use log::{debug, warn};
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use thiserror::Error;
use ultimate_mod_man_rs_utils::{
    types::{ModId, VariantAndId},
    user_input_delegate::UserInputDelegate,
};

use crate::utils::{
    FuzzyMatchedStr, FuzzySearchMatchRes,
    fuzzy_search_strings_and_return_one_or_many_depending_on_perfect_match,
};

pub type BananaScraperResult<T> = Result<T, BananaScraperError>;

static BANANA_ROOT: &str = "https://gamebanana.com";

/// I think this is hard coded into the web interface and can't be changed?
const NUM_SEARCH_RESULTS_PER_PAGE: usize = 15;

#[derive(Debug, Error)]
pub enum BananaScraperError {
    #[error(
        "Unable to find the mod \"{0}\". Make sure that the mod name is an exact match (including \
         case) to the mod name on GameBanana."
    )]
    ModNameNotFound(String),

    #[error("The mod variant {0} was not found for the mod {1}.")]
    ModVariantDoesNotFound(String, String),

    #[error(
        "Got a different MD5 checksum for the artifact {0} of the mod {1}: (Expected: {2}, Ours: \
         {3}). This file has likely been tampered with!"
    )]
    VariantMd5CheckSumMismatch(String, String, String, String),

    #[error(transparent)]
    JsonDeserializationError(#[from] serde_json::Error),

    #[error("Error during Reqwest client initialization")]
    ClientError(#[from] reqwest::Error),
}

#[derive(Debug)]
pub struct ScrapedBananaModData {
    pub mod_name: String,
    pub variant_name: String,
    pub version: Option<String>,
    pub variant_download_artifact: Vec<u8>,
}

#[derive(Debug)]
pub struct BananaClient {
    client: Client,
}

impl BananaClient {
    pub fn new() -> BananaScraperResult<Self> {
        let client = ClientBuilder::default().build()?;

        Ok(Self { client })
    }

    async fn get_mod_page_for_mod_id(&self, name: &str) -> BananaScraperResult<String> {
        todo!()
    }

    pub async fn resolve_mod_name(&self, name: &str) -> BananaScraperResult<ModId> {
        debug!("Resolving mod name \"{}\" to it's ID...", name);

        let search_req = format!(
            "{}/apiv11/Util/Search/Results?_sModelName=Mod&_sOrder=best_match&_idGameRow=6498&\
             _sSearchString={}&_csvFields=name&_nPage=1",
            BANANA_ROOT, name
        );
        let search_resp: SearchResp =
            serde_json::from_str(&self.client.get(search_req).send().await?.text().await?)?;

        // We are going to enforce that searching by a name MUST match the name of the
        // mod exactly, including case.
        match search_resp
            .a_records
            .into_iter()
            .find(|item| item.s_name == name)
        {
            Some(entry) => Ok(entry.id_row),
            None => {
                if search_resp.a_meta_data.n_record_count > NUM_SEARCH_RESULTS_PER_PAGE {
                    warn!(
                        "Unable to find an exact mod name match for \"{}\" on GameBanana. We only \
                         searched through {} entries but {} were matched by the search in \
                         general. Make sure that the name of the mod is entered exactly the way \
                         it's spelled (including case) on GameBanana.",
                        name, NUM_SEARCH_RESULTS_PER_PAGE, search_resp.a_meta_data.n_record_count
                    );
                }

                Err(BananaScraperError::ModNameNotFound(name.to_string()))
            },
        }
    }

    pub async fn download_mod_variant(
        &self,
        user_input_delegate: &mut impl UserInputDelegate,
        key: &VariantAndId,
    ) -> BananaScraperResult<ScrapedBananaModData> {
        debug!("Downloading mod {}...", key);

        let mod_page_req = format!("{}/apiv11/Mod/{}/ProfilePage", BANANA_ROOT, key.id);
        let mod_page_resp: ModPageResp =
            serde_json::from_str(&self.client.get(mod_page_req).send().await?.text().await?)?;

        // We're not going to require an exact match here, but will use fuzzy matching
        // instead.
        let mod_file_names = mod_page_resp
            .a_files
            .iter()
            .map(|file| file.s_file.as_str())
            .collect::<Vec<_>>();

        let match_idx = match fuzzy_search_strings_and_return_one_or_many_depending_on_perfect_match(
            &mod_file_names,
            &key.variant_name,
        ) {
            FuzzySearchMatchRes::Perfect(idx) => idx,
            FuzzySearchMatchRes::Multiple(sorted_matches) => {
                let sorted_matches_massaged = sorted_matches
                    .into_iter()
                    .map(|x| MatchedFileVariant {
                        variant_name: mod_file_names[x.idx],
                        intern: x,
                    })
                    .collect::<Vec<_>>();

                user_input_delegate.select_item_from_list(&sorted_matches_massaged)
            },
            FuzzySearchMatchRes::None => {
                return Err(BananaScraperError::ModVariantDoesNotFound(
                    key.variant_name.to_string(),
                    mod_page_resp.s_name,
                ));
            },
        };

        let version = mod_page_resp
            .s_version
            .is_empty()
            .then_some(mod_page_resp.s_version);

        let selected_variant = &mod_page_resp.a_files[match_idx];
        let payload_download_url = selected_variant.s_download_url.as_str();
        let variant_download_artifact = self
            .client
            .get(payload_download_url)
            .send()
            .await?
            .bytes()
            .await?
            .to_vec();

        // Verify that the MD5 hash matches (idk why they are using MD5 instead od
        // something like SHA256...)
        let calculated_md5 = format!("{:x}", md5::compute(&variant_download_artifact));
        if calculated_md5 != selected_variant.s_md5_checksum {
            return Err(BananaScraperError::VariantMd5CheckSumMismatch(
                selected_variant.s_file.clone(),
                mod_page_resp.s_name,
                selected_variant.s_md5_checksum.clone(),
                calculated_md5,
            ));
        }

        Ok(ScrapedBananaModData {
            mod_name: mod_page_resp.s_name,
            variant_name: selected_variant.s_file.clone(),
            version,
            variant_download_artifact,
        })
    }
}

#[derive(Debug)]
struct MatchedFileVariant<'a> {
    intern: FuzzyMatchedStr,
    variant_name: &'a str,
}

impl Display for MatchedFileVariant<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} -- ({:3})", self.variant_name, self.intern.score.0)
    }
}

#[derive(Debug, Deserialize)]
struct SearchResp {
    #[serde(rename = "_aMetadata")]
    a_meta_data: SearchMetaData,

    #[serde(rename = "_aRecords")]
    a_records: Vec<SearchRespItem>,
}

#[derive(Debug, Deserialize)]
struct SearchMetaData {
    #[serde(rename = "_nRecordCount")]
    n_record_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchRespItem {
    #[serde(rename = "_idRow")]
    id_row: ModId,

    #[serde(rename = "_sName")]
    s_name: String,
}

#[derive(Debug, Deserialize)]
struct ModPageResp {
    #[serde(rename = "_sName")]
    s_name: String,

    #[serde(rename = "_sVersion")]
    s_version: String,

    #[serde(rename = "_aFiles")]
    a_files: Vec<ModDownloadEntries>,
}

#[derive(Debug, Deserialize)]
struct ModDownloadEntries {
    #[serde(rename = "_sFile")]
    s_file: String,

    #[serde(rename = "_tsDateAdded")]
    ts_date_added: u64,

    #[serde(rename = "_sDownloadUrl")]
    s_download_url: String,

    #[serde(rename = "_sMd5Checksum")]
    s_md5_checksum: String,
}
